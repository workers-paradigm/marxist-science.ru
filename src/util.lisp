;;;; util.lisp

(in-package #:marxist-science)

(defvar *user* nil)
(defvar *acceptor* nil)
(defvar *dbpass* nil)
(defvar *db-connection* nil)

(defun resource-path (path)
  (asdf:system-relative-pathname :marxist-science path))

(setf *dbpass* (getf (uiop:read-file-form (resource-path "secrets.lisp")) :dbpass))

(defun start ()
  (when (and *acceptor* (tbnl:started-p *acceptor*))
    (tbnl:stop *acceptor*))
  (setf *acceptor* (make-instance 'easy-routes:easy-routes-acceptor
                                  :port 4242
                                  :document-root (truename (resource-path "pub"))
                                  :error-template-directory (truename (resource-path "pub/err"))))
  (setf tbnl:*show-lisp-errors-p* t)
  (tbnl:start *acceptor*))

(defun stop ()
  (tbnl:stop *acceptor*)
  (pomo:clear-connection-pool))


(defmacro with-pooled-connection (&body body)
  `(pomo:with-connection `("http" "http" ,*dbpass* "localhost" :pooled-p t)
     ,@body))

(defun hash-password (password &optional (salt (crypto:random-data 32)))
  (format
     nil "~A:~A" (crypto:byte-array-to-hex-string salt)
     (crypto:byte-array-to-hex-string
      (crypto:derive-key
       (crypto:make-kdf 'crypto:scrypt-kdf :n (expt 2 14))
       (crypto:ascii-string-to-byte-array password)
       salt
       (expt 2 14)
       32))))

(defun get-salt (combination)
  (subseq combination 0 (position #\: combination)))

(defun verify-password (password hash-combination)
  (let ((salt (get-salt hash-combination)))
    (string= hash-combination (hash-password password (crypto:hex-string-to-byte-array salt)))))

(defun session-user ()
  "Returns the id of the user with the session from cookies.
Advances the expiration date by one month."
  (rtl:when-let (session (tbnl:cookie-in "session"))
    (rtl:when-let (session-user (tbnl:cookie-in "user-id"))
      (with-pooled-connection
        (let ((query (pomo:prepare "SELECT id, user_id FROM valid_sessions WHERE content = $1 AND user_id = $2" :plist)))
          (rtl:when-it (funcall query session session-user)
            (funcall (pomo:prepare "CALL delay_session_expiration ($1)") (getf rtl:it :id))
            (getf rtl:it :user-id)))))))

(defun log-out ()
  (rtl:when-let (session (tbnl:cookie-in "session"))
    (with-pooled-connection
      (let ((query (pomo:prepare "DELETE FROM sessions WHERE content = $1")))
        (funcall query session)))))

(defun log-in (password &optional (user "admin"))
  "Returns a list (SESSION USER-ID EXPIRES) if the log in was succesful, or NIL if not.
Creates a session entry for this user in the database."
  (with-pooled-connection
    (let ((query (pomo:prepare "SELECT id, pass_hash FROM users WHERE name = $1" :list)))
      (destructuring-bind (id hash) (or (funcall query user) (return-from log-in))
        (when (verify-password password hash)
          (let ((session (crypto:byte-array-to-hex-string (crypto:random-data 64))))
            (funcall (pomo:prepare "SELECT * FROM new_session($1, $2)" :list)
                     id session)))))))

(defun upload-file (filespec)
  "FILESPEC: (PATH &OPTIONAL NAME MIMETYPE)
Uploads file from PATH to server and returns (values ID URL) corresponding to the file."
  (destructuring-bind (path &optional (name "") (mimetype "")) filespec
    (with-pooled-connection
      (let ((md5sum (crypto:byte-array-to-hex-string (md5:md5sum-file path)))
            (file-exists-p (pomo:prepare "SELECT id, url FROM files WHERE md5 = $1" :list))
            (upload-file (pomo:prepare "INSERT INTO files (url, md5) VALUES ($1, $2)" :none)))
        (rtl:when-it (funcall file-exists-p md5sum)
          (return-from upload-file (values-list rtl:it)))
        (let* ((extension (or (get-ext mimetype) (pathname-type path) (pathname-type name)))
               (new-fname (format nil "~A.~A" md5sum extension))
               (url (format nil "/uploads/~A" new-fname)))
          (uiop:copy-file path (resource-path (format nil "pub/uploads/~A" new-fname)))
          (funcall upload-file url md5sum)
          (values-list (funcall file-exists-p md5sum)))))))

(defun delete-uploaded-file (url)
  (uiop:delete-file-if-exists (resource-path (format nil "pub~A" url))))

(defun replace-file (file-id new-file-spec)
  "FILESPEC: (PATH &OPTIONAL NAME MIMETYPE)
Uploads the new file and updates the DB entry with the given `FILE-ID' to hold the new
location and md5sum of the file, and deletes the old file, if needed."
  (destructuring-bind (path &optional (name "") (mimetype "")) new-file-spec
    (with-pooled-connection
      (let ((md5sum (crypto:byte-array-to-hex-string (md5:md5sum-file path)))
            (get-url (pomo:prepare "SELECT url FROM files WHERE id = $1" :single))
            (same-file-p (pomo:prepare "SELECT url FROM files WHERE id = $1 AND md5 = $2" :single))
            (replace-file (pomo:prepare "UPDATE files SET url = $1, md5 = $2 WHERE id = $3" :none)))
        (when (funcall same-file-p file-id md5sum)
          (return-from replace-file))
        (let* ((extension (or (get-ext mimetype) (pathname-type path) (pathname-type name)))
               (new-fname (format nil "~A.~A" md5sum extension))
               (url (format nil "/uploads/~A" new-fname)))
          (uiop:copy-file path (resource-path (format nil "pub/uploads/~A" new-fname)))
          (delete-uploaded-file (funcall get-url file-id))
          (funcall replace-file url md5sum file-id))))))
