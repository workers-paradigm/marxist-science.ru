;;;; page-templates.lisp

(in-package #:marxist-science)
(in-readtable sushiroller)

(defparameter %url-location-pairs
  '(("/" "главная")
    ("/articles" "статьи")
    ("/archive" "архив")))

(defparameter %head
  #@@@(progn
   #@(meta :charset "UTF-8")
   #@(meta :name "viewport" :content "width=device-width,initial-scale=1")
   #@(link :rel "stylesheet" :href "/css/styles.css" :type "text/css")))

(defmacro standard-page (params &body body)
  (destructuring-bind (&key (title "") (scripts ())) params
    `#@@@
    (progn
      #@@ (format nil "<!DOCTYPE html>")
      #@ (html
          (head
           @ %head
           (title @ ,title))
          (body
           (ul.site-menu
            @@ (mapc
                (lambda (pair)
                  #@(li (a :href @(car pair) @(cadr pair))))
                %url-location-pairs))
           (div.content
            @@ (progn ,@body)
            (footer
             (hr)
             (ul.user-menu
              "Меню пользователя:"
              @@ (if *user*
                     #@(li (a :href "/logout" "Выйти"))
                     #@(li (a :href "/login" "Войти"))))
             (p.copyright-notice
              "Copyright &copy; 2022 Marxist Science | All rights reserved.")))
           @@ (mapc (lambda (path) #@(script :src @path :defer "defer")) ,scripts))))))

(defun edit-page (&key (title "") (article-content "Содержание статьи тут") id)
  (standard-page (:title "написать статью | НМ" :scripts '("/js/upload.js"))
    #@(section.write-article
       (h1.heading "написать статью")
       (form#article-writer
        :action "/api.articles.publish" :method "POST" :enctype "multipart/form-data"
        @@(when id #@(input :type "hidden" :name "id" :value @id))
        (input :type "text" :name "title" :placeholder "Название" :value @title)
        (label :for "cover" "Обложка:")
        (input :id "cover" :type "file" :name "image")
        (textarea :name "content" @article-content)
        (input :type "submit" :value "Опубликовать")))
    #@(section.upload
       (ul.upload-files
        (form#upload-form
         :action "/api.upload" :method "POST" :enctype "multipart/form-data"
         (span.error#upload-error :style "display:none;")
         (div.input-group
          (input :type "file" :accept "image/*" :name "fileToUpload")
          (input :type "submit" :value "Загрузить файл")))))))
