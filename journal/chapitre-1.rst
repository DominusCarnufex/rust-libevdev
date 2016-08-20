Ce premier chapitre sera relativement long, et je m’en excuse d’avance. Comme toujours quand on débute l’apprentissage d’un nouveau langage, il y a beaucoup de choses à voir d’un coup pour obtenir un résultat concret même minimal. C’est d’autant plus vrai ici qu’il faut *également* découvrir libevdev.

Alors n’attendons pas plus.

.. contents::

Bonjour, la rouille !
=====================

Avant d’envisager quoi que ce soit d’autre, il vous faut installer le compilateur Rust et la bibliothèque standard. Vous trouverez tout le nécessaire sur la `page de téléchargement`__ du site officiel.

.. __: https://www.rust-lang.org/en-US/downloads.html

Prenez bien garde, cependant, d’utiliser la version Nightly ! La programmation système utilise des outils qui sont encore considérés comme instables et ne sont donc pas disponibles dans les versions Stable et Bêta du compilateur. Dès que l’installation est faite, vous pouvez entrer la commande suivante pour vérifier que tout s’est bien passé.

.. code:: console

    carnufex@KAMARAD-PC $ rustc -V
    rustc 1.12.0-nightly (080e0e072 2016-08-08)

Et afin de se frotter sans attendre au langage, voici un programme minimal.

.. code:: rust

    fn main()   {
        println!("Bonjour, la rouille !"); // « Hello, Rust! », pour ceux qui
                                    // n’auraient pas fait le rapprochement.
    }

Pour compiler, utilisez la commande suivante.

.. code:: console

    rustc -O -C prefer-dynamic -o test bonjour.rs

Vous ne serez pas surpris en apprenant que l’argument ``-O`` permet d’appliquer autant que possible les optimisations à la compilation, ni que l’argument ``-o`` sert à définir le nom du programme compilé. En l’absence de celui-ci, le programme porte le même nom que le fichier source, sans le ``.rs``.

L’option ``-C prefer-dynamic`` indique à rustc de lier dynamiquement les bibliothèques à l’exécutable (et en particulier la bibliothèque standard). En effet, Rust est de ces langages qui, par défaut, lient les bibliothèques statiquement.

Outre que cela fait exploser la taille des exécutables (649 Ko ici, au lieu de 9 Ko), cela devient complètement ridicule quand on écrit une bibliothèque : si l’on écrit plusieurs bibliothèques, elles seront toutes liées statiquement aux mêmes bibliothèques, chacune de leur côté. Cet argument vous sera donc bien utile.

Testez. Profitez. Et puis passons à l’analyse du code lui-même.

Comme vous connaissez un peu de C, tout devrait vous être relativement familier. Les commentaires, tout d’abord : ``//`` pour un commentaire sur une seule ligne, et la paire ``/*`` et ``*/`` pour un commentaire sur plusieurs lignes. Le fait d’écrire une chaîne de caractères entre guillemets doubles, ensuite, même si l’analogie s’arrête là. Le point-virgule à la fin d’une instruction, enfin, qui connaît cependant une exception (on la verra en temps utile).

Par ailleurs, tout exécutable se doit d’avoir une fonction ``main``, qui ne prend pas d’argument et ne renvoie rien. D’où la syntaxe très simple, avec le mot-clé ``fn``, les parenthèses vides, et les accolades qui délimitent le bloc de la fonction.

Pour finir, ``println`` n’est pas une fonction, mais une **macro**, ainsi que l’indique le ``!`` après son nom. Les macros de Rust n’ont pas grand chose à voir avec celles de C, et leur fonctionnement interne est une notion assez avancée, donc on va laisser cela de côté. La bonne nouvelle, c’est qu’il n’y a pas besoin d’importer quoi que ce soit pour bénéficier de quelque chose d’aussi universellement utile.

Pour votre culture, il existe aussi une macro ``print`` qui fait tout comme ``println``, mais sans le retour à la ligne à la fin.

À l’attaque !
=============

Ceci étant dit, par où commence-t-on quand on veut remplacer une bibliothèque fondamentale de son système d’exploitation ? La première idée qui vient à l’esprit est `la page officielle`__ de ladite bibliothèque. Et c’est ici une bonne idée. En effet, on y trouve un lien vers la documentation, un autre vers le code source de la dernière version en date, et même un code d’exemple de la manière de l’utiliser.

.. __: https://www.freedesktop.org/wiki/Software/libevdev/

Il est évidemment indispensable de télécharger le code source de la bibliothèque, et d’installer *a minima* les fichiers d’en-tête ``libevdev.h`` et ``libevdev-uinput.h`` à un endroit où votre compilateur C saura les trouver. Selon votre distribution, vous pouvez également installer le paquet ``libevdev-dev`` ou quel que soit son nom, mais cela ne vous dispense pas d’avoir le code source complet sous la main.

Quant au code d’exemple, vous allez vite vous rendre compte qu’il est inutilisable en l’état. En effet, il manque la fonction ``main``, ansi que tous les en-têtes, et le code utilise des fonctions qui n’existent pas. Voici donc une version légèrement et salement modifiée, qui compile, elle.

.. code:: c

    #include <errno.h>
    #include <fcntl.h>
    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>

    #include <libevdev.h>

    int main()  {
        struct libevdev *dev = NULL;
        int fd;
        int rc = 1;
        fd = open("/dev/input/event6", O_RDONLY|O_NONBLOCK);
        rc = libevdev_new_from_fd(fd, &dev);
        if (rc < 0) {
            fprintf(stderr, "Failed to init libevdev (%s)\n", strerror(-rc));
            exit(1);
        }
        printf("Input device name: \"%s\"\n", libevdev_get_name(dev));
        printf("Input device ID: bus %#x vendor %#x product %#x\n",
           libevdev_get_id_bustype(dev),
           libevdev_get_id_vendor(dev),
           libevdev_get_id_product(dev));
        if (!libevdev_has_event_type(dev, EV_REL) ||
            !libevdev_has_event_code(dev, EV_KEY, BTN_LEFT))
        {
            printf("This device does not look like a mouse\n");
            exit(1);
        }
        do {
            struct input_event ev;
            rc = libevdev_next_event(dev, LIBEVDEV_READ_FLAG_NORMAL, &ev);
            if (rc == 0)
                printf("Event: %s %s %d\n",
                   "[placeholder]",
                   "[placeholder]",
                   ev.value);
        } while (rc == 1 || rc == 0 || rc == -EAGAIN);

        return 0;
    }

Un petit point s’impose avant d’aller plus loin. Dans les grandes lignes, ce code ouvre le fichier ``/dev/input/event6``, appelle toute une série de fonctions de libevdev pour mettre en place la communication avec le périphérique et affiche leur résultat, puis affiche quelques informations sur chacun des événements renvoyés par le périphérique.

Il faut savoir que, chaque fois qu’un périphérique d’entrée est branché sur votre ordinateur, evdev crée un fichier de type *device* dans ``/dev/input/`` pour le représenter. Celui-ci portera le nom ``eventN``, en fonction de l’ordre dans lequel les périphériques ont été détectés.

Pour ceux qui ne seraient pas familiers du procédé, dans les systèmes d’exploitation de la famille Unix, tout est un fichier. Les périphériques, la sortie standard (``stdout``), les processus en cours d’exécution, la mémoire vive, et même le néant intersidéral (``/dev/null``). Ils sont *tous* représentés par des fichiers, situés essentiellement dans ``/dev/`` et ``/proc/``.

De cette manière, pour le programmeur, interagir avec l’un de ces éléments « abstraits » ne consiste en rien d’autre qu’à ouvrir le fichier qui le représente (``open()``), lire et écrire dedans (``read()`` et ``write()``) et/ou réaliser un IOCTL dessus (en très gros, c’est une lecture-écriture dopée aux stéroïdes), puis le fermer (``close()``). Ce qui est évidemment *extrêmement* pratique.

Le nom exact du fichier qui représente la souris *chez vous* n’est donc pas nécessairement ``event6``, et vous devrez modifier le code en circonstance. Généralement, ``/dev/input/`` contient un sous-dossier ``by-path`` avec des liens vers les fichiers *device* ayant un nom plus clair : cherchez celui ou ceux qui se terminent par ``-event-mouse``, et voyez sur quel ``eventN`` il pointe.

À présent, je vous laisse compiler comme des grands. Le résultat du programme saute aux yeux : au moindre déplacement de votre souris, aussi minime soit-il, une ligne apparaît sur la sortie standard, avec deux ``[placeholder]`` et un nombre, correspondant au déplacement de votre souris dans une direction ou une autre.

.. important::

    Pour des raisons évidentes de sécurité, n’importe qui ne peut pas accéder à la porte d’entrée directe vers le noyau, et vous devrez lancer votre programme avec les pouvoirs d’administrateur pour qu’il s’exécute correctement.

    De plus, seul un ``Ctrl + C`` permettra d’arrêter le programme.

Ce simple code d’exemple permet déjà de comprendre quelques petites choses sur le fonctionnement de libevdev. En premier lieu, la fonction d’entrée dans libevdev est ``libevdev_new_from_fd``, qui à partir du descripteur de fichier correspondant au fichier *device*, renvoie un ``struct libevdev``. Cet objet doit ensuite être passé à toutes les fonctions subséquentes de libevdev, en lieu et place du descripteur de fichier.

En deuxième lieu, il existe de nombreuses fonctions comme ``libevdev_get_name`` et ``libevdev_has_event_type`` qui servent simplement à interroger le périphérique pour obtenir des informations à son sujet.

En troisième lieu, chaque fois que le périphérique est manipulé dans le monde extérieur, evdev génère un événement, qui peut être récupéré au moyen de la fonction ``libevdev_next_event`` sous la forme d’un type ``struct input_event``.

Il y a bien évidemment beaucoup d’autres fonctions et possibilités dans la bibliothèque, mais cela nous donne un point de départ et un angle d’attaque, dont il sera toujours temps de s’écarter plus tard. Je vous invite donc à présent à aller parcourir le code source de la bibliothèque, pour voir par vous-mêmes comment il est organisé, et en particulier comment fonctionnent dans les grandes lignes les fonctions utilisées par le code ci-dessus.

C’est fait ?

C’est bien. Voici les premières observations qui peuvent être faites après une lecture superficielle du code (pas besoin de plus pour l’instant).

- Comme c’est souvent le cas avec les bibliothèques système de Linux, l’API exposée par le noyau est composée exclusivement d’IOCTL, dont on peut trouver la liste complète dans le dossier ``include/`` et en particulier dans ``input.h``.
- Le fichier ``libevdev-uinput.c`` et les autres fichiers qui lui sont associés semblent créer une surcouche à libevdev même, que nous pouvons donc oublier dans un premier temps.
- Il existe en réalité deux points d’entrée dans la bibliothèque : soit créer un ``struct libevdev`` avec ``libevdev_new`` puis l’initialiser avec ``libevdev_set_fd``, soit passer directement par ``libevdev_new_from_fd`` qui fait les deux d’un coup. En outre, en principe, on est censé faire appel à ``libevdev_free`` avant de quitter la bibliothèque, pour nettoyer derrière soi.
- Le type ``struct libevdev`` est défini dans ``libevdev-int.h`` et est une structure très, très touffue.
- Très peu de fonctions de la bibliothèque font des IOCTL : les fonctions ``sync_*_state``, une ou deux autres fonctions du même type, et surtout ``libevdev_set_fd`` dont on vient de parler.

De tout cela, on en déduit que l’organisation générale est la suivante : la fonction ``libevdev_set_fd`` réalise la plupart des IOCTL, qui sont coûteux, une bonne fois pour toutes, et place le résultat dans un des nombreux champs de ``struct libevdev`` ; de cette manière, les fonctions visant à interroger le périphérique vont simplement vérifier les champs de ``struct libevdev``, opération nettement moins coûteuse ; l’essentiel des autres IOCTL correspond aux occasions où l’on ne peut pas s’en passer, en particulier lorsqu’il faut resynchroniser le ``struct libevdev`` et l’état réel du périphérique.

Eh bien voilà ! Nous savons désormais par où commencer !

Interrogation timide
====================

Pour avoir un semblant de résultat qu’il nous sera possible de comparer avec le programme en C, nous devons parvenir à faire deux choses avec notre programme en Rust.

1. Ouvrir le fichier *device* correspondant à la souris.
2. Réaliser le ``printf`` de la ligne 20, c’est-à-dire implémenter les trois fonctions ``libevdev_get_id_*``.

Le premier objectif est indépendant de libevdev. Quant au second, le fichier ``libevdev.c`` nous apprend que ces trois fonctions (et une quatrième qui n’apparaît pas dans le code) renvoient simplement un des quatre sous-champs du champ ``ids`` du ``struct libevdev``.

Le fichier ``libevdev-int.h`` nous apprend que ce champ ``ids`` est de type ``struct input_id``, lequel type est à son tour défini dans le fichier ``include/linux/input.h`` comme suit.

.. _struct-input_id:

.. code:: c

    struct input_id {
	    __u16 bustype;
	    __u16 vendor;
	    __u16 product;
	    __u16 version;
    };

Enfin, c’est naturellement dans la fonction ``libevdev_set_fd`` que ce champ est rempli en premier lieu, et plus spécifiquement grâce à la ligne suivante.

.. code:: c

	rc = ioctl(fd, EVIOCGID, &dev->ids);

Et quid de ``EVIOCGID`` ? Il est lui aussi défini dans ``include/linux/input.h``, sous la forme de cette macro.

.. code:: c

    #define EVIOCGID        _IOR('E', 0x02, struct input_id)

Oui, c’est laid, et cela ne nous apprend pas grand chose. Si vous allez fouiller dans l’en-tête ``sys/ioctl.h`` du noyau Linux, vous trouverez la définition de la macro ``_IOR``, qui s’appuie elle-même sur une autre macro, bref, on a déjà perdu trop de temps pour si peu. Utilisez le code suivant, et voyez ce qui s’affiche à l’exécution (*spoil* : c’est ``0x80084502``).

.. code:: c

    #include <stdio.h>
    #include <sys/ioctl.h>

    #define EVIOCGID        _IOR('E', 0x02, struct input_id)

    int main()  {
        printf("EVIOCGID = %#x\n", EVIOCGID);
        return 0;
    }

Vous pourrez bien évidemment réemployer ce procédé pour tous les IOCTL que vous rencontrerez au cours de cette aventure.

.. note::

    Si ce n’est pas encore fait, apprenez de toute urgence à utiliser le programme ``grep`` : il sera votre meilleur ami pour retrouver où une fonction, une macro ou une variable a été définie au milieu de tous les fichiers d’un code source, ou encore où elles sont employées ailleurs que là où vous les avez rencontrées la première fois.

Objectif nº 1 : ouvrir un fichier *device*
==========================================

Rust a été explicitement pensé pour pouvoir interagir facilement avec du code écrit en C. Cela se traduit par le fait qu’un pan entier de la bibliothèque standard du langage est destiné à faire appel aux fonctions de la bibliothèque standard de C. Ce pan s’appelle ``libc``, et pour pouvoir l’utiliser, il vous faudra ajouter la ligne suivante au début de votre code source.

.. code:: rust

    extern crate libc;

La syntaxe est d’un usage plus général que pour le simple cas de ``libc``, mais il est trop tôt pour parler du système de modules. Essayez de compiler votre programme. C’est un échec. Je l’ai fait exprès pour vous montrer un aspect fort appréciable de Rust : les erreurs de compilation sont *détaillées*. Voici ce qu’on obtient dans le cas présent.

.. code:: console

    evdev.rs:1:1: 1:19 error: use of unstable library feature 'libc': use `libc` from crates.io (see issue #27783) 
    evdev.rs:1 extern crate libc;
               ^~~~~~~~~~~~~~~~~~
    evdev.rs:1:1: 1:19 help: add #![feature(libc)] to the crate attributes to enable 
    error: aborting due to previous error 

Il y a même, comme souvent, une suggestion de solution à apporter, et nous allons l’adopter, parce que c’est la bonne. Ajoutez cette ligne tout en haut de votre code source.

.. code:: rust

    #![feature(libc)]

À présent, toutes les fonctions, tous les types et toutes les constantes de ``libc`` sont disponibles pour votre code source. Mais comme elles viennent d’un *crate* différent (Rust appelle ses paquets des *crates*), elles sont rangées dans un espace de noms spécifique, portant le même nom que le *crate*. La syntaxe pour y accéder est ``<crate>::<identifiant>``.

Il est possible de déplacer l’un ou l’autre de ces identifiants dans l’espace de noms général, à l’aide de l’instruction que voici.

.. code:: rust

    use <crate>::<identifiant>;

Si on veut importer plusieurs identifiants d’un coup, on peut le faire au moyen de la syntaxe ``use <crate>::{<id1>, <id2>, <id3>};``, voire utiliser ``use <crate>::*;`` pour tous les importer. Ce qui dans le cas de ``libc`` serait très hasardeux.

C’est une affaire de goût, mais de manière générale, je décourage l’importation d’un quelconque identifiant de ``libc`` à part les types : je préfère rester clair quant au fait qu’il s’agit de fonctions C, donc potentiellement dangereuses.

Et naturellement, pour ouvrir un fichier, nous allons nous intéresser à la fonction ``libc::open``, qui prend comme argument un ``*const libc::c_char`` (un pointeur nu constant vers des ``char`` de C, donc) représentant le chemin d’accès au fichier, et un ``libc::c_int`` représentant les drapeaux. Elle renvoie un ``libc::c_int``.

Dans votre fonction ``main``, vous allez donc ajouter la ligne suivante.

.. code:: rust

    let fd = libc::open(0 as *const c_char, libc::O_RDONLY | libc::O_NONBLOCK);

Vous avez certainement deviné ce qui se passe dans cette ligne, mais on va quand même tout expliquer pour être sûr de ne rien rater.

La syntaxe ``let <identifiant> = <expression>;`` est utilisée pour déclarer une variable. La convention veut qu’un identifiant de variable soit en *snake_case*, et si vous ne respectez pas cette convention, le compilateur va rouspéter.

Notez également que, par défaut, une variable est non mutable. Le code suivant ne compilera pas.

.. code:: rust

    let a = 2;
    a = 42;

Si vous voulez que votre variable soit mutable, il faudra utiliser la syntaxe ``let mut`` à la place. Sachez que rendre une variable mutable sans nécessité absolue est considérée comme une mauvaise pratique, et que rustc produira un avertissement si une variable mutable n’est jamais modifiée par le code.

Deuxième point à noter, il est possible de déclarer une variable sans l’initialiser. Cependant, rustc vous attend au tournant, et refusera de compiler si vous essayez de lire le contenu de cette variable sans qu’il soit absolument certain qu’elle a nécessairement été initialisée avant cette lecture. De cette manière, Rust empêche l’accès à des données qui seraient dans un état indéterminé.

Troisième point, on peut également déclarer explicitement le type de la variable au moment de sa définition. La syntaxe est la suivante.

.. code:: rust

    let <identifiant> : <type> = …

Mais Rust utilise l’inférence de types, et il n’est généralement pas nécessaire de préciser le type d’une variable, alors profitez-en pour alléger le code.

Reste un dernier morceau de syntaxe à expliquer : ``0 as *const c_char``. L’idée est bien évidemment de passer un pointeur ``NULL`` à la fonction. Mais en l’absence d’autre indication, et si l’inférence de type ne pointe pas vers un autre type numérique, une valeur immédiate entière est considérée comme un ``i32``, c’est-à-dire un entier signé sur 32 bits.

Et un pointeur n’est pas un type numérique en Rust. Il est donc obligatoire de faire une conversion de types explicite, ce qui s’accomplit au moyen du mot-clé ``as``, sans difficulté particulière.

Essayez donc de compiler. Vous obtenez l’erreur suivante.

.. code:: console

    evdev.rs:9:14: 10:56 error: call to unsafe function requires unsafe function or block [E0133]
    evdev.rs:9     let fd = libc::open(0 as *const c_char,
                            ^
    evdev.rs:9:14: 10:56 help: run `rustc --explain E0133` to see a detailed explanation

Là encore, c’était fait exprès, pour pouvoir vous montrer votre nouveau meilleur ami : la commande ``rustc --explain``. La plupart des erreurs de compilation peuvent vous être expliquées en détail si vous le souhaitez, et c’est un très bon complément à l’apprentissage du langage lui-même.

Dans le cas qui nous intéresse, le problème vient du fait que ``libc::open`` est une fonction qui a été déclarée comme ``unsafe`` dans sa définition. On l’a déjà dit, Rust essaye d’apporter le plus de sécurité possible au code, et pour ce faire, il offre la possibilité de déclarer explicitement qu’une fonction *n’est pas* sûre et doit être utilisée avec précaution.

Il existe deux situations dans lesquelles vous pouvez faire appel à des éléments ``unsafe`` du langage. Soit dans le code d’une fonction que vous avez déclarée comme ``unsafe`` : on verra comment faire en temps utile. Soit en entourant les passages incriminés d’un bloc ``unsafe { … }``. Notre ligne de tout à l’heure devrait donc s’écrire ainsi.

.. code:: rust

    let fd = unsafe { libc::open(0 as *const c_char, libc::O_RDONLY | libc::O_NONBLOCK) };

Reste un problème : ouvrir un pointeur ``NULL``, cela ne nous amènera rien de bon. Il faudrait passer le nom du fichier *device* que l’on veut ouvrir. Seulement, les chaînes de caractères en Rust n’ont presque rien à voir avec les chaînes de caractères en C.

Dans ce dernier langage, une chaîne de caractères est une suite de ``char`` contigus, c’est-à-dire d’entiers sur 8 bits, dont le dernier vaut 0, ou ``'\0'`` comme on peut l’écrire de manière plus claire. En Rust, les ``char`` sont encodés en UTF-8, et occupent pour cette raison 32 bits. Il existe par ailleurs deux types de chaînes de caractères.

Lorsque vous fournissez une valeur immédiate de type chaîne, par exemple ``"/dev/input/event6"``, celle-ci a pour type ``&str``. Il est trop tôt pour expliquer en quoi cela consiste exactement, mais l’important à retenir, c’est que c’est un type natif du langage, et qu’il *n’est pas* terminé par un ``'\0'`` comme en C. Il est très hasardeux de partir du principe que l’octet suivant la chaîne en mémoire sera bien un 0, et qu’on peut donc le passer sans danger à ``libc::open``.

L’autre type de chaînes est le type ``String``. Lui est un objet défini dans la bibliothèque standard, et sa représentation interne est bien cachée. Surtout, contrairement à ``&str``, il existe un moyen simple d’ajouter un caractère à la fin de la chaîne.

Toutes les manipulations que nous allons effectuer seront faites au moyen de méthodes que la bibliothèque standard a définies pour les types ``&str`` et ``String``. Leur syntaxe d’appel ne surprendra personne, puisqu’il s’agit de ``<objet>.<méthode>(<arguments>)``. Nous allons en utiliser trois ici.

- ``to_string()`` est une méthode de ``&str``, qui renvoie un ``String`` contenant strictement la même chaîne de caractères.
- ``push(<caractère>)`` est une méthode de ``String``, qui ajoute un caractère à la fin de la chaîne. Cela n’a pas été précisé jusqu’à présent, mais un caractère s’écrit entre guillemets simples, comme dans la plupart des langages. Le gros défaut de cette méthode, c’est qu’elle ne renvoie rien, au lieu de renvoyer le ``String`` modifié : il n’est ainsi pas possible de chaîner les ``push``, ni d’appliquer directement d’autre méthodes sur le ``String`` modifié.
- ``as_ptr()`` est une méthode disponible pour les deux types, qui renvoie un pointeur nu sur les données brutes de la chaîne de caractères. Le type exact est ``*const u8``, et vous aurez deviné que ``u8`` désigne un entier non signé sur 8 bits.

Je vous laisse à présent essayer de trouver par vous-mêmes le code complet de l’ouverture de notre fichier *device*. Je mets la solution en-dessous, mais ne regardez pas tout de suite.

.. code:: rust

    let mut st = "/dev/input/event6".to_string();
    st.push('\0');
    let pt = st.as_ptr() as *const c_char;
    let fd = unsafe { libc::open(pt, libc::O_RDONLY | libc::O_NONBLOCK) };

Objectif nº 2 : un IOCTL à la mer
=================================

Pour cette deuxième étape, on va naturellement recourir à la fonction ``libc::ioctl``. Elle prend en entrée un ``libc::c_int`` qui représente le descripteur de fichier sur lequel on va faire l’IOCTL, un ``libc::c_ulong`` qui représente l’identifiant unique de l’IOCTL et… d’éventuels arguments supplémentaires de types indéfinis. Je pense que vous commencez à imaginer quelle va être la difficulté.

La valeur de retour de cette fonction est un ``libc::c_int``, aucune difficulté là-dedans. Et bien évidemment, la fonction est *totalement* ``unsafe``.

Alors pour commencer, histoire de voir si vous avez assimilé ce que l’on a vu jusqu’à présent, vous allez écrire l’appel à la fonction ``libc::ioctl``, en ne mettant pas de troisième argument, mais en mettant les bonnes valeurs pour les besoins de notre code. Il faut bien sûr que votre code compile.

.. code:: rust

    let io = unsafe { libc::ioctl(fd, 0x80084502 as c_ulong) };

S’il y a quelque chose là-dedans que vous ne comprenez pas, retournez lire les sections précédentes, tout y est expliqué. Et rien ne sert d’aller plus loin si ces notions ne sont pas acquises.

Reste la question du troisième argument. Dans le code C, il s’agit d’un pointeur vers une variable de type ``struct input_id`` préalablement déclarée, dont le contenu sera fourni par l’IOCTL. En Rust, il va donc falloir commencer par définir un type équivalent à ``struct input_id``.

Si vous ne vous souvenez plus de la définition du type en C, elle se trouve dans `la section 4`__. En Rust, voici comment il faudra le définir.

.. __: struct-input_id_

.. code:: rust

    #[repr(C)]
    struct InputId  {
	    bustype : u16,
	    vendor  : u16,
	    product : u16,
	    version : u16
    }

À nouveau, beaucoup de nouveautés. Alors prenons-les dans un ordre logique. Tout d’abord, la syntaxe générale pour déclarer une structure est la suivante.

.. code:: rust

    struct <identifiant du type> {
        [<identifiant> : <type du champ>,]*
    }

La dernière virgule est optionnelle, il est très important de *ne pas* mettre de point-virgule à la fin de la définition, et la convention veut que les identifiants de type soient en *CamelCase* (et rustc rouspète toujours si vous ne la respectez pas).

Pour chaque champ, la présence du type est *obligatoire*. Tous les types ne peuvent pas être utilisés comme champs d’une structure, mais on verra cela en temps utile. Ici, les quatre champs sont des ``u16``, soit des entiers non signés sur 16 bits, ce qui correspond strictement au type ``__u16`` utilisé en C.

Enfin, il faut savoir que rustc fait un peu ce qu’il veut avec la représentation exacte en mémoire de ses structures. Pour être certain qu’il n’y touchera pas, et que la structure aura exactement la même représentation en mémoire que si elle avait été définie dans un programme en C, il faut ajouter la ligne ``#[repr(C)]`` avant le début de la définition.

Dernier point à noter : cette définition de type doit se trouver *en dehors* de la fonction ``main``, même si son emplacement exact est sans importance.

Passons maintenant à l’étape suivante : créer une variable de type ``InputId`` dans notre fonction ``main``, et initialiser tous ses champs à 0. Le code est le suivant.

.. code:: rust

    let mut ii = InputId    {
	    bustype : 0,
	    vendor  : 0,
	    product : 0,
	    version : 0
    };

Comme vous le voyez, la similarité avec la définition du type est frappante. Et malheureusement, il n’existe pas de solution plus économique pour mettre tous les champs de la structure à 0. Si vous décidez de ne pas l’initialiser, rustc vous enverra paître quand vous voudrez lire le contenu des champs après l’IOCTL, parce que *lui* ne sait pas que l’IOCTL a initialisé les champs (``unsafe``, tout ça…).

Vous noterez que la variable est mutable. En effet, l’IOCTL va bien être obligé de modifier le contenu des champs, donc cela doit lui être permis.

Il ne reste plus qu’à passer cette variable ``ii`` à ``libc::ioctl``, sous la forme d’un pointeur nu. Cela va se faire en deux étapes.

Premièrement, on va créer une référence mutable vers ``ii``. La syntaxe est très simple, c’est ``&mut ii``. Vous devinerez sans peine qu’il est possible de faire des références non mutables avec la syntaxe ``&<identifiant>``. Oui, cela fait furieusement penser à ``&str``, et non, ce n’est pas un hasard, et non, je n’expliquerai pas plus en détail : les références sont un aspect très important de Rust, on y reviendra en temps utile.

Deuxièmement, on va convertir cette référence en un pointeur nu, mutable lui aussi, au moyen de la syntaxe ``as``. Le type de ce pointeur nu sera ``*mut _``. Pourquoi ``_`` ? Parce qu’on ignore totalement le type exact qu’attend la fonction en C (il s’agit certainement d’un ``void *``), alors, ben… on ne s’embête pas à le préciser. Voici, en conséquence, le code complet de l’appel à ``libc::ioctl``.

.. code:: rust

    let io = unsafe {
        libc::ioctl(fd, 0x80084502 as c_ulong, &mut ii as *mut _)
    };

J’aimerais vous faire remarquer deux choses.

Tout d’abord, la fonction marche tout aussi bien si vous utilisez une variable, une référence et un pointeur non mutables (``*const _`` pour ce dernier). En effet, la fonction C qui reçoit l’appel ignore tout de Rust et de ses histoires de mutabilité, et pour elle, un pointeur est un pointeur. C’est pour les gens qui liront votre programme que vous devez mettre tous ces ``mut`` : sans cela, ils ne peuvent pas deviner au premier coup d’œil que l’IOCTL va modifier le contenu de ``ii``.

Ensuite, la conversion en pointeur nu n’est pas absolument nécessaire. Si vous essayez avec juste la référence, cela va compiler et s’exécuter correctement. Seulement, une référence est plus qu’un pointeur, elle contient aussi des informations sur la taille de l’objet qu’elle pointe, et il est donc *dangereux* de présumer que la fonction C va lire exactement ce qu’il faut en mémoire.

Pour terminer de réaliser l’objectif, il ne nous reste plus qu’à afficher les trois valeurs qui nous ont été retournées par l’IOCTL.

Vous vous souvenez certainement de la macro ``println``. Elle est nettement plus proche de ``printf`` que ce que vous en avez vu pour l’instant. Il est ainsi possible de placer des « balises » dans la chaîne de caractères à afficher, représentées par ``{}``, qui seront remplacées par les valeurs ou variables passées en arguments supplémentaires.

Par exemple, ce code…

.. code:: rust

    println!("Je vais compter de {} à {}.", 1, 42);

… affichera ce résultat.

.. code:: console

    Je vais compter de 1 à 42.

Et il est possible de personnaliser un peu ces balises. Par exemple, avec ``{:x}``, si un nombre est passé en argument, il sera affiché en notation hexadécimale. Et avec ``{:p}``, c’est l’adresse du pointeur ou de la référence passés en argument qui sera affichée, plutôt que la valeur de la variable pointée.

Quant à accéder aux champs d’une structure préexistante, cela se fait à l’aide d’une syntaxe qui ne vous surprendra pas le moins du monde, à savoir ``<identifiant>.<champ>``. Je vous laisse à présent essayer de terminer ce programme, et vérifier que les résultats obtenus sont bien les mêmes qu’avec le programme en C. La solution est ci-dessous, mais prenez le temps d’y réfléchir un peu avant de regarder.

.. code:: rust

    #![feature(libc)]

    extern crate libc;
    use libc::{c_char, c_ulong};

    #[repr(C)]
    struct InputId  {
	    bustype : u16,
	    vendor  : u16,
	    product : u16,
	    version : u16
    }

    fn main()   {
        let mut st = "/dev/input/event6".to_string();
        st.push('\0');
        let pt = st.as_ptr() as *const c_char;
        let fd = unsafe { libc::open(pt, libc::O_RDONLY | libc::O_NONBLOCK) };

        let mut ii = InputId    {
	        bustype : 0,
	        vendor  : 0,
	        product : 0,
	        version : 0
        };

        let io = unsafe {
            libc::ioctl(fd, 0x80084502 as c_ulong, &mut ii as *mut _)
        };

        println!("Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
            ii.bustype, ii.vendor, ii.product);
    }

--------

Nous voilà rendus au bout de ce premier chapitre. Notre programme ne fait pas encore grand chose, et surtout, Rust ne doit pas vous sembler si différent de C, à part quelques règles de sécurité supplémentaires. Mais rassurez-vous, vous sentirez mieux la différence après avoir lu le prochain chapitre.
