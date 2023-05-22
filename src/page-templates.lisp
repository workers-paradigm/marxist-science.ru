;;;; page-templates.lisp

(in-package #:marxist-science)

(defparameter %head
  '((meta :charset "UTF-8")
    (meta :name "viewport" :content "width=device-width,initial-scale=1")
    (link :rel "stylesheet" :href "/css/styles.css" :type "text/css")))

(defparameter %url-location-pairs
  '(("/" "главная")
    ("/articles" "статьи")
    ("/archive" "архив")))

(defun standard-page (params &rest body)
  (destructuring-bind (&key (title "") (script-paths ())) params
    (jhtml:jhtml
     '(jhtml:doctype)
     `(html
       (head
        ,@%head
        (title ,title))
       (body
        (ul :class "site-menu"
            ,@ (mapcar
                (lambda (pair)
                  `(li (a :href ,@pair)))
                %url-location-pairs))
        (div :class "content"
          ,@body
          (footer
           (hr)
           (ul :class "user-menu"
               "Меню пользователя:"
               ,@ (if (not *user*)
                      '((li (a :href "/login" "Войти")))
                      '((li (a :href "/logout" "Выйти")))))
           (p :class "copyright-notice"
              "Copyright" (jhtml:insert-html " &copy; ")
              "2022 Marxist Science | All rights reserved.")))

        ,@ (mapcar (lambda (path) `(script :src ,path :defer t))
                   script-paths))))))

(defun edit-page (&key (title "") (article-content "Содержание статьи тут") id)
  (standard-page
   '(:title "написать статью | НМ"
     :script-paths ("/js/upload.js"))
   `(section :class "write-article"
     (h1 :class "heading" "написать статью")
     (form :id "article-writer" :action "/api.articles.publish" :method "POST" :enctype "multipart/form-data"
      ,(when id `(input :type "hidden" :name "id" :value ,id))
      (input :type "text" :name "title" :placeholder "Название" :value ,title)
      (label :for "cover" "Обложка:")
      (input :id "cover" :type "file" :name "image")
      (textarea :name "content" ,article-content)
      (input :type "submit" :value "Опубликовать")))
   '(section :class "upload"
     (ul :id "upload-files"
      (form :id "upload-form" :action "/api.upload" :method "POST" :enctype "multipart/form-data"
       (span :class "error" :style "display:none;" :id "upload-error")
       (div :class "input-group"
        (input :type "file" :accept "image/*" :name "fileToUpload")
        (input :type "submit" :value "Загрузить файл")))))))
