;;;; marxist-science.asd

(asdf:defsystem #:marxist-science
  :description "Сайт интернет-журнала «Наука Марксизм»."
  :license  "BSD 3 Clause"
  :version "0.0.1"
  :depends-on (#:hunchentoot
               #:easy-routes
               #:jhtml
               #:rutils
               #:postmodern
               #:ironclad
               #:cl-json)

  :serial t
  :components
  ((:file "package")
   (:module "src"
    :serial t
    :components
    ((:file "mime")
     (:file "util")
     (:file "page-templates")
     (:file "marxist-science")))))
