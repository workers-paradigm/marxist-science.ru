(in-package :marxist-science)
(in-readtable sushiroller)

(defroute homepage ("/" :decorators (@html @auth)) ()
  (standard-page (:title "Наука марксизм")
   #@(section.about
      (h1.heading (span.highlight "Наука марксизм"))
      (p "Наука марксизм — коммунистический интернет-журнал, целью которого является развитие марксистской
         теории, пропаганда и агитация диалектического и исторического материализма, политической
         экономии и научного социализма.")
      (p "Ресурс "
         (span.highlight "«Наука Марксизм»")
         " является печатным органом коммунистической кружковой организации. Редакция НМ обеспечивает работу
          по ряду направлений:")
      (p "Во-первых, это научно-исследовательская работа, отдельные редакторы или научно-исследовательские
         группы занимаются этой работой в соответствии с генеральной линией.")
      (p "Во-вторых, это образовательный процесс, в рамках которого позиции группы доводятся до слушателей
         кружков.")
      (p "В-третьих, это агитационная работа, направленная на распространение марксистско-ленинских воззрений
         вширь и вглубь."))))

(defroute login ("/login" :decorators (@html @auth)) ()
  (when *user*
    (return-from login (tbnl:redirect "/")))
  (standard-page (:title "Вход | Наука Марксизм")
    #@(section.login
       (h1 "Вход")
       (form
        :action "/api.authenticate" :method "POST"
        (input :type "password" :name "password" :placeholder "Пароль")
        (input :type "submit" :value "Войти")))))

(defroute articles ("/articles" :method :get :decorators (@html @auth)) ()
  (standard-page (:title "статьи | НМ")
   #@(section#sectors
      (h1.heading (span.highlight "статьи по разделам"))
      (div.sector))))

(defun list->div.manage-sector (list)
  (destructuring-bind (id name cover-url) list
    #@(div.manage-sector :id @id
      @@(unless (eq cover-url :null) #@(img :src @cover-url))
      ;; TODO image upload and sector settings form
      (div.manage-applet
       (div.header
        (span @name))))))

(defroute manage ("/manage" :method :get :decorators (@html @auth-required)) ()
  (with-pooled-connection
    (standard-page (:title "Правка | НМ" :scripts '("/js/interactive.js"))
      #@(section#manage
         (h1.heading (span.highlight "разделы/правка"))
         @@ (mapc #'list->div.manage-sector (pomo:query "SELECT * FROM sectors_names_covers" :lists))))))
