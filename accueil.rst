Le module **evdev**, abréviation de *event device*, est un morceau du noyau Linux. Il prend en charge les « entrées » de manière générale, même si son champ exact est un peu flou. Il gère les claviers, les souris, les *joysticks*, les tablettes graphiques, les écrans tactiles et même le fait que le bouton d’allumage de l’ordinateur ait été enfoncé. En revanche, il ne s’occupe pas des caméras, des micros, ou des entrées des périphériques réseau.

Si un programme a besoin de communiquer avec un de ces périphériques — par exemple, un jeu vidéo dirigé au *joystick* — la personne qui le développe utilisera généralement la bibliothèque **libevdev**, ou un autre programme qui utilise lui-même libevdev. Comme la quasi-totalité des bibliothèques système, elle est écrite en C.

Le projet Rust libevdev a pour but de réécrire cette bibliothèque en Rust. L’idée n’est bien évidemment pas de calquer le code C et son fonctionnement exact, mais d’écrire une bibliothèque qui joue le même rôle que libevdev, tout en offrant une API plus sûre et plus pratique que l’actuelle, voire ayant un fonctionnement globalement plus sécurisé.

Par ailleurs, cela constitue une bonne occasion de faire découvrir Rust à ceux qui ne le connaîtraient pas encore. En particulier, de montrer aux amateurs de programmation système que Rust est un concurrent sérieux pour C et C++. C’est pourquoi, tout au long du développement, j’écrirai une série d’article visant à enseigner Rust à ceux qui connaissent déjà C et s’intéressent à la programmation système.

L’introduction de ce « journal de développement » se trouve `ici`__, et voici la liste des chapitres déjà écrits.

.. __: {{ site.baseurl }}/journal

{% include liste-chapitres.html %}

À chaque chapitre du journal est associée une branche du dépot GitHub, appelée ``chapitreN``, contenant l’état du code complet à l’issue de ce chapitre. Vous pouvez la télécharger à l’aide des boutons situés en haut de chaque chapitre. La branche ``master``, quant à elle, contient la dernière version en date.

.. important::

    Le lien vers la documentation situé en haut à droite ne fonctionne pas encore, le code n’étant pas assez avancé pour justifier d’être documenté.

Contribuer
==========

Dans la mesure où le développement suit la logique du journal de bord, qui a lui-même une visée pédagogique, proposer des fonctionnalités supplémentaires ou une méthode plus efficace de coder un passage donné n’est sans doute pas pertinent. N’hésitez cependant pas à faire des retours, par le biais des *issues* de GitHub ou de l’adresse de contact située plus bas, dans un des cas suivants.

- Le code proposé ne compile pas, où bien ne donne pas le résultat attendu.
- Il y a une faute d’orthographe ou de typographie dans le texte ou quelque part sur le site.
- Vous avez une suggestion pertinente pour la suite du journal et vous êtes d’avis que je n’y ai certainement pas pensé.
- Vous ne comprenez pas un passage du journal et avez besoin d’explications supplémentaires.
- Vous adorez mon travail et voulez connaître mon numéro de compte en Suisse pour me faire un virement non imposable.

Par ailleurs, il n’y a aucune raison que seuls les francophones puissent découvrir la puissance de Rust en programmation système. Par conséquent, vous pouvez proposer des traductions dans d’autres langues, aussi bien du site que du journal (et de la documentation quand elle existera).

Pour cela, vous pouvez télécharger la source de chaque page à l’aide du bouton situé en haut de celle-ci, et la modifier à votre convenance. Elle utilise une version légèrement modifiée de `ReStructuredText`__ : ne vous étonnez donc pas si le rendu n’est pas le même chez vous que sur le site.

.. __: https://aful.org/wikis/interop/ReStructuredText

Mentions légales
================

- Le code de la bibliothèque est diffusé sous `licence CeCIll-B version 1`__. Pour ceux qui ne connaîtraient pas, il s’agit d’une licence similaire à la licence MIT.
- Les textes du site et le journal de bord sont diffusés sous licence `BiPu L`__. Il s’agit d’une licence proche d’une licence Creative Commons BY-NC-SA.
- La documentation, lorsqu’elle existera, sera diffusée sous licence `Bien Public`__. Cela est aussi proche du domaine public que la législation française le permet.
- Le thème du site est largement inspiré du thème « Hack » de GitHub Pages, statut légal inconnu.
- Le thème de coloration syntaxique est un clone du thème « Cobalt » de GtkSourceView, adapté à un fond noir.
- Les icônes des boites spéciales — comme le bloc « Important » ci-dessus — sont piquées à `Zeste de Savoir`__.

.. __: http://www.cecill.info/licences/Licence_CeCILL-B_V1-fr.html
.. __: http://www.teladiai.re/public/licences/BiPu_L.pdf
.. __: http://www.teladiai.re/public/licences/BiPu.pdf
.. __: https://zestedesavoir.com/

Pour me contacter si vous n’avez pas de compte GitHub ou si vous ne souhaitez pas l’utiliser, vous pouvez m’écrire à l’adresse ``dοmіnuѕ.саrnufеx@tеlаdіаі.rе``. Attention ! Ne faites pas de copier-coller : j’utilise une `technique de fourbe`__ pour tromper les robots, et l’adresse ne fonctionnera pas si vous ne la tapez pas vous-même.

.. __: http://altf4.teladiai.re/index.php?post/23
