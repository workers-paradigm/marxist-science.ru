;;;; package.lisp

(defpackage #:marxist-science
  (:use #:cl)
  (:import-from :easy-routes
                #:defroute
                #:@json
                #:@html)
  (:import-from :named-readtables #:in-readtable)
  (:import-from :sushiroller #:sushiroller))
