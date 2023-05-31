;;;; marxist-science.lisp

(in-package #:marxist-science)

(defun @auth (next)
  (let ((*user* (session-user)))
    (funcall next)))

(defun @auth-required (next)
  (let ((*user* (session-user)))
    (if *user*
        (funcall next)
        (tbnl:redirect "/login"))))

(defroute logout ("/logout" :decorators ()) ()
  (log-out)
  (tbnl:redirect "/"))


;;; API
(defroute api.authenticate ("/api.authenticate" :method :post :decorators (@auth)) (&post password)
  (tbnl:redirect
   (rtl:cond-it
     ((null password) "/login")
     (*user* "/")
     ((log-in password)
      (destructuring-bind (session user-id expires) rtl:it
        (tbnl:set-cookie "session" :value session :expires expires)
        (tbnl:set-cookie "user-id" :value user-id :expires expires)
        "/"))
     (t "/login"))))

(defun publish-new (title image content)
  (uiop:with-temporary-file (:pathname path :stream stream :type "md" :direction :output)
    (format stream "~A" content)
    :close-stream
    ;; TODO: Uploaded image filename might not have an extension
    (let ((img-id (upload-file image))
          (md-id (upload-file (list path))))
      (with-pooled-connection
        (let* ((query (pomo:prepare "SELECT * FROM insert_article ($1, $2, $3)" :single))
               (id (funcall query title md-id img-id)))
          (tbnl:redirect (format nil "/articles/view?p=~D" id)))))))

(defun replace-article (id title image content)
  (with-pooled-connection
    (destructuring-bind (image-id markdown-id)
        (funcall (pomo:prepare "SELECT cover, markdown FROM articles WHERE id = $1" :list) id)
      (uiop:with-temporary-file (:pathname path :stream stream :type "md" :direction :output)
        (format stream "~A" content)
        :close-stream
        ;; TODO: update existing file records, not insert new ones
        (funcall (pomo:prepare "UPDATE articles SET title = $1 WHERE id = $2" :none) title id)
        (replace-file markdown-id (list path))
        (when image (replace-file image-id image))
        (tbnl:redirect (format nil "/articles/view?p=~D" id))))))

(defroute api.articles.publish ("/api.articles.publish" :method :post :decorators (@auth-required))
    (title image content (article-id :real-name "id"))
  (if article-id
      (replace-article article-id title image content)
      (publish-new title image content)))

(defroute api.upload ("/api.upload" :method :post :decorators (@json @auth-required)) ((file-to-upload :real-name "fileToUpload"))
  (let ((output-plist (list :content-url "" :error "")))
    (flet ((err! (code message)
             (setf (tbnl:return-code*) code)
             (setf (getf output-plist :error) message)
             (return-from api.upload (json:encode-json-plist-to-string output-plist))))
      (unless *user*
        (err! tbnl:+http-forbidden+ "Not authorized to upload"))
      (unless file-to-upload
        (err! tbnl:+http-bad-request+ "No valid file provided"))
      (multiple-value-bind (id url) (upload-file file-to-upload)
        (declare (ignore id))
        (setf (getf output-plist :content-url) url)
        (json:encode-json-plist-to-string output-plist)))))
