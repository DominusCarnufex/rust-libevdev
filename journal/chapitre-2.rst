Dans ce deuxième chapitre, nous n’allons pas ajouter de nouvelle fonctionnalité à notre programme : à l’issue de ce chapitre, il se contentera toujours d’afficher la même ligne qu’avant. Cependant, le code sera nettement différent, car il fera appel à plusieurs des outils de Rust pour avoir un code plus propre et plus sûr.

.. contents::

Même C le fait
==============

La sécurité minimale que peut offrir un programme, c’est de vérifier si la valeur de retour d’une fonction correspond à ce qui était attendu. Le code d’exemple en C le fait ici.

.. code:: c

    rc = libevdev_new_from_fd(fd, &dev);
    if (rc < 0) {
        fprintf(stderr, "Failed to init libevdev (%s)\n", strerror(-rc));
        exit(1);
    }

De manière plus surprenante, il ne vérifie pas que la fonction ``open`` a fonctionné correctement. En revanche, le code de libevdev vérifie bien que chaque IOCTL a bien tourné, comme ici, pour le cas qui nous intéresse.

.. code:: c

	rc = ioctl(fd, EVIOCGID, &dev->ids);
	if (rc < 0)
		goto out;

Le moins que l’on puisse faire est d’effectuer la même vérification dans notre code Rust. Et la bonne nouvelle, c’est que la syntaxe pour un bloc conditionnel est quasiment identique en Rust et en C.

.. code:: rust

    if <cond>   {

    } else if <cond>    {

    } else  {

    }

Les accolades sont naturellement obligatoires, mais contrairement à C, Rust ne met pas de parenthèses autour de ses conditions. Rust utilise également les même opérateurs de comparaison (``==``, ``!=``, ``<``, etc.) et de combinaison de conditions (``&&`` et ``||``) que C, vous ne serez pas dépaysés.

La question qui demeure, c’est : que mettre dans ce bloc conditionnel ? Rust est très pratique de ce point de vue-là. En effet, il existe une macro ``panic`` qui fonctionne exactement comme ``println``, à ceci près que le programme abandonne en affichant le message d’erreur.

Ainsi, si on ajoute les quelques lignes suivantes après l’ouverture du fichier…

.. code:: rust

    if fd < 0   {
        panic!("Impossible d’ouvrir le fichier {}.", st);
    }

… on obtient le résultat suivant quand on tente de lancer le programme sans les droits d’administrateur.

.. code:: console

    thread 'main' panicked at 'Impossible d’ouvrir le fichier /dev/input/event6.', libevdev.rs:21
    note: Run with `RUST_BACKTRACE=1` for a backtrace.

La fonction ``ioctl`` fonctionne sur le même modèle que ``open``, et renvoie ``-1`` en cas d’échec. Je vous laisse donc implémenter la vérification vous-mêmes. Pour tester et voir si votre code fonctionne, passez ``0`` comme argument de l’IOCTL à la place de ``0x80084502``.

Mais le système d’erreur des fonctions C ne se limite pas à renvoyer ``-1`` en cas de problème. Il existe une variable ``errno`` accessible globalement et qui contient un code correspondant à la dernière erreur rencontrée. Seulement, Rust ne permet pas d’y accéder directement.

Il y a dans le *crate* ``libc`` une fonction ``__errno_location()`` qui renvoie un pointeur nu vers l’endroit où se trouve ``errno``. Vous commencez à avoir l’habitude, cette fonction est ``unsafe``. En outre, comme elle renvoie un pointeur nu, il faut déréférencer ce dernier pour obtenir la valeur. Ce qui est aussi ``unsafe``, et se fait au moyen de l’opérateur ``*``, comme on pouvait s’y attendre.

Voici donc une version légèrement plus complète de notre bloc de vérification pour ``open``.

.. code:: rust

    if fd < 0   {
        panic!("Impossible d’ouvrir le fichier {} (errno = 0x{:x}).", st, unsafe { *libc::__errno_location() });
    }

Cette fois, si vous lancez le programme sans droits d’administrateur, vous obtenez le résultat suivant.

.. code:: console

    thread 'main' panicked at 'Impossible d’ouvrir le fichier /dev/input/event6 (errno = 0xd).', libevdev.rs:21
    note: Run with `RUST_BACKTRACE=1` for a backtrace.

L’erreur ``0xd``, c’est ``EACCESS``, ce qui ne vous surprendra pas. En l’état, cependant, ce n’est pas très évident au premier abord. On pourrait ajouter un traitement avec ``strerror``, mais la vérité, c’est que les messages d’erreur renvoyés ne sont absolument pas informatifs.

On va donc plutôt mettre cela dans un coin, et saisir l’occasion d’en apprendre un peu plus sur les fonctions Rust. Vous connaissez déjà la fonction ``main``, qui ne prend aucun argument et ne renvoie rien. Ici, nous allons voir comment renvoyer un argument.

La première chose à faire, c’est de fournir le type de la valeur de retour dans la définition de la fonction. C’est à dire que, entre la parenthèse fermante et l’accolade ouvrante, on va ajouter ``->`` suivi du type de retour. Comme ceci.

.. code:: rust

    fn errno() -> c_int {
        unimplemented!();
    }

Notez la macro ``unimplemented`` : elle permet que votre programme compile même si vous n’avez pas encore terminé d’implémenter le corps d’une fonction. En revanche, si vous tentez de l’exécuter, vous aurez une erreur.

La deuxième étape, c’est de renvoyer la bonne valeur. Comme en C, le mot-clé ``return`` permet de faire exactement cela.

.. code:: rust

    fn errno() -> c_int {
        return unsafe { *libc::__errno_location() };
    }

Mais ce n’est pas la manière idiomatique de faire. En effet, si la dernière instruction dans un bloc *n’est pas* terminée par un point-virgule, alors cette instruction représente la valeur de retour de ce bloc. C’est notamment le cas avec les fonctions : le code suivant est parfaitement équivalent au précédent.

.. code:: rust

    fn errno() -> c_int {
        unsafe { *libc::__errno_location() }
    }

Vous pouvez dès lors appeler ``errno()`` dans votre message de panique. Ce que nous n’allons pas faire, parce que cela n’apporte aucune information pertinente. Mais si l’on n’utilise pas la fonction, rustc râle.

.. code:: console

    warning: function is never used: `errno`, #[warn(dead_code)] on by default

Le moyen de faire taire rustc, c’est d’ajouter ``#[allow(dead_code)]`` immédiatement avant la définition de la fonction incriminée (cela marche aussi avec les structures et autres définitions de types que nous verrons plus tard). Une autre manière de faire (et qui fonctionne avec absolument tous les identifiants), c’est d’ajouter un ``_`` au début du nom de la fonction (``_errno()``).

Enfin, si vous ajoutez ``#![allow(dead_code)]`` (notez le point d’exclamation) au tout début de votre code, vous ne recevrez plus *aucun* avertissement de ce type. C’est rarement une bonne idée.

En guise de conclusion à cette section, je vous donne un petit exercice. Écrivez une fonction ``new_input_id`` qui renvoie un ``InputId`` initialisé à 0 partout, et utilisez-la dans le code de ``main``. La définition de la fonction est juste en-dessous, attendez avant de regarder.

.. code:: rust

    fn new_input_id() -> InputId    {
        InputId {
	        bustype : 0,
	        vendor  : 0,
	        product : 0,
	        version : 0
        }
    }

Des fonctions… Des fonctions partout…
=====================================

Il est temps de passer à la vitesse supérieure et d’avoir des fonctions qui peuvent prendre des arguments. Vous vous souvenez de la syntaxe ``<identifiant> : <type>`` utilisée dans les déclarations de variable ? Eh bien elle sert aussi pour déclarer les arguments d’une fonction. Voici par exemple une fonction qui prend deux entiers non signés de 32 bits et renvoie la somme de leurs carrés.

.. code:: rust

    fn inutile(a : u32, b : u32) -> u32 {
        a * a + b * b
    }

.. note::

    Comme cela devient pénible d’écrire « entiers non signés de 32 bits » à chaque fois, voici une liste des types numériques de Rust.

    - Entiers signés : ``i8``, ``i16``, ``i32``, ``i64``, ``isize``.
    - Entiers non signés : ``u8``, ``u16``, ``u32``, ``u64``, ``usize``.
    - Flottants : ``f32``, ``f64``.

    Les types ``isize`` et ``usize`` ont une taille différente selon la plate-forme : ils ont la taille nécessaire pour contenir un pointeur.

Ceci étant dit, la question du passage d’arguments à une fonction est moins triviale qu’il n’y paraît. En effet, Rust applique une sémantique de déplacement à toutes les liaisons de variable. Prenez le code suivant.

.. code:: rust

    struct Point    {
        x : u16,
        y : u16
    }

    fn main()   {
        let p1 = Point { x : 42, y : 79 };
        let p2 = p1;

        println!("({}, {})", p1.x, p1.y);
    }

Si vous essayez de le compiler, vous obtiendrez la double erreur suivante.

.. code:: console

    error[E0382]: use of moved value: `p1.x`
    error[E0382]: use of moved value: `p1.y`

Cela vient du fait qu’à tout moment, il ne peut y avoir qu’un et un seul identifiant associé à une donnée particulière. Ainsi, la première ligne de ``main`` associe l’identifiant ``p1`` à la donnée ``Point { x : 42, y : 79 }``. Puis la seconde ligne associe ``p2`` à cette même donnée, et coupe la liaison entre elle et ``p1``. Si bien qu’arrivés à la quatrième ligne, ``p1`` n’est plus associé à aucune donnée, et tenter d’accéder à un de ses champs ne peut rien donner de bon.

Pour placer un peu de vocabulaire, on dit qu’une variable a la **propriété** d’une donnée lorsque ladite donnée et l’identifiant de la variable sont liés, et l’on dit que l’on **déplace** (*move*) la valeur lorsque l’on transfère sa propriété d’une variable à une autre.

Cette construction, qui peut paraître pénible au premier abord, poursuit plusieurs objectifs.

Premièrement, à tout moment, le compilateur sait quelle variable est propriétaire de quelle donnée. Cela signifie que lorsque cette variable n’est plus utilisée (par exemple, parce qu’on arrive au bout de la fonction où elle a été définie), le compilateur peut supprimer la donnée sans risque. On limite ainsi fortement les fuites de mémoire.

Mais surtout, contrairement à la plupart des langages, cette vérification de la durée de vie des données est réalisée à la compilation et non à l’exécution, ce qui permet de se passer d’un ramasse-miettes, et ainsi d’accélérer sensiblement l’exécution.

Deuxièmement, cela facilite l’écriture de programmes parallélisés, car une même donnée ne peut pas être utilisée par deux fils d’exécution à la fois : un seul de ces fils en a la propriété, et s’il transmet la donnée à un autre fil, il ne peut plus s’en servir lui-même.

Mais revenons-en à nos fonctions. Lorsque l’on passe une valeur en argument d’une fonction, on crée une liaison entre la valeur et l’argument, et la valeur n’est plus accessible dans la fonction appelante. Le code suivant ne compilera pas non plus, et avec les mêmes erreurs.

.. code:: rust

    struct Point    {
        x : u16,
        y : u16
    }

    fn affiche(p : Point)   {
        println!("({}, {})", p.x, p.y);
    }

    fn main()   {
        let p1 = Point { x : 42, y : 79 };

        affiche(p1);

        println!("({}, {})", p1.x, p1.y);
    }

Il faudrait que la fonction appelée rende à la fonction appelante la propriété de ces données en les renvoyant, comme ceci.

.. code:: rust

    struct Point    {
        x : u16,
        y : u16
    }

    fn affiche(p : Point) -> Point  {
        println!("({}, {})", p.x, p.y);
        p
    }

    fn main()   {
        let mut p1 = Point { x : 42, y : 79 };

        p1 = affiche(p1);

        println!("({}, {})", p1.x, p1.y);
    }

Mais une telle solution devient très vite très pénible, outre qu’elle nous oblige à rendre ``p1`` mutable pour qu’il puisse récupérer sa propre valeur. C’est pourquoi Rust permet de passer une variable par **référence**, ce qui s’appelle **emprunter** la variable.

Une référence est signalée par la présence de ``&`` au début de l’identifiant du type ou de la variable. Elle a précisément pour caractéristique de *ne pas* prendre la propriété de la donnée concernée. Voici donc un code légèrement modifié, pour que notre fonction se contente d’emprunter la variable.

.. code:: rust

    struct Point    {
        x : u16,
        y : u16
    }

    fn affiche(p : &Point)  {
        println!("({}, {})", p.x, p.y);
    }

    fn main()   {
        let p1 = Point { x : 42, y : 79 };

        affiche(&p1);

        println!("({}, {})", p1.x, p1.y);
    }

Naturellement, lorsqu’une valeur a été empruntée, il est interdit de réaliser sur elle une quelconque action qui nécessite d’en transférer la propriété. Le code suivant, par exemple, ne compilera pas.

.. code:: rust

    struct Point    {
        x : u16,
        y : u16
    }

    fn affiche(p : Point)   {
        println!("({}, {})", p.x, p.y);
    }

    fn main()   {
        let p1 = Point { x : 42, y : 79 };
        let p2 = &p1;

        affiche(p1);

        println!("({}, {})", p2.x, p2.y);
    }

En effet, ``affiche`` tente de prendre la propriété de la valeur liée à ``p1`` alors qu’elle est empruntée par ``p2``, et qu’elle le reste jusqu’à la disparition de ``p2``, à la fin du bloc.

.. _copy:

.. note::

    Vous aurez remarqué que tous les exemples utilisent une structure ``Point`` plutôt que quelque chose de plus simple, comme un ``u32``. C’est qu’il existe un mécanisme, que nous verrons plus tard, qui permet de demander que les valeurs d’un type donné soient copiées plutôt que déplacées. Et notamment, tous les types numériques natifs ont cette propriété, de même que les pointeurs nus.

Il reste encore quelques petites choses à voir pour comprendre comment fonctionnent les références et ne pas se retrouver face à des erreurs de compilation cryptiques.

La première, c’est qu’une référence a une **durée de vie**, c’est-à-dire une portion du code dans laquelle elle est valide et pointe de manière certaine vers des données qui existent, et cette durée de vie ne peut naturellement pas être supérieure à celle des données elles-mêmes. Prenez l’exemple suivant.

.. code:: rust

    fn main()   }

        let y : &i32;

        {
            let x = 42;
            y = &x;
        }

        println!("{}", y);

    }

La compilation échoue avec le message d’erreur suivant.

.. code:: console

    error: `x` does not live long enough

En effet, ``x`` est déclarée entre les accolades, et disparaît donc à la fin de ce bloc. Ce qui fait qu’au moment de l’appel à ``println``, ``y`` pointe vers des données qui n’existent plus, et le compilateur ne le permet pas.

.. important::

    Les variables sont supprimées dans l’ordre inverse de celui où elles ont été créées. Cela signifie que si vous créez une variable de type référence avant la variable sur laquelle elle va pointer, vous obtiendrez la même erreur que ci-dessus.

La deuxième chose, c’est qu’il est possible de demander qu’un argument soit mutable dans la fonction : il suffit de placer le mot-clé ``mut`` avant son nom. De la même manière, on peut emprunter une valeur dans l’optique de la modifier, au moyen d’une référence mutable (``&mut <identifiant>``). Évidemment, la variable de départ doit être mutable, sinon, ce serait trop facile.

Il y a cependant une règle très importante. **Dans un même bloc, il ne peut exister qu’une seule référence mutable à une même valeur, et des références mutables et non mutables ne peuvent cohabiter.** Par exemple, le code suivant refuse de compiler (``println`` utilise des références non mutables en sous-main).

.. code:: rust

    let mut x = 42;
    let y = &mut x;

    println!("{}", x);

Pour que le code compile, il faut s’assurer que la référence mutable ait disparu avant toute utilisation d’une référence non mutable, ce qui peut se faire en la mettant dans un bloc créé pour l’occasion.

.. code:: rust

    let mut x = 42;

    {
        let y = &mut x;
    }

    println!("{}", x);

Là aussi, cela peut sembler pénible, mais cela permet de paralléliser des programmes en s’assurant qu’une donnée ne sera pas modifiée par un fil d’exécution pendant qu’un autre essaye de la lire. Ou que l’on ne va pas ajouter des éléments à un tableau que l’on est en train de parcourir. Etc.

Le troisième point, c’est qu’une référence peut être déréférencée afin d’accéder à la valeur pointée, au moyen du même opérateur ``*`` que pour les pointeurs nus. Voyez par exemple ce code, qui permet de modifier ``x`` par le biais de sa référence ``y``.

.. code:: rust

    let mut x = 40;

    {
        let y = &mut x;
        *y += 2;
    }

    println!("{}", x);

Voilà, cela fait beaucoup d’informations d’un coup, et il reste encore beaucoup à dire, notamment sur les durées de vie. Mais il est indispensable de bien comprendre ces mécanismes de propriété et d’emprunt pour ne pas s’arracher les cheveux sur des erreurs en apparence incompréhensibles, et ils sont indissociables les uns des autres.

Pour terminer en douceur par un petit exercice, vous allez écrire une fonction qui prend un ``String`` et le transforme en chaîne de caractères C (``*const c_char``), et l’utiliser à la place des premières lignes de la fonction ``main``. Comme d’habitude, la solution est en-dessous, mais prenez le temps de réfléchir.

.. code:: rust

    fn to_c_string(st : &mut String) -> *const c_char   {
        st.push('\0');
        st.as_ptr() as *const c_char
    }

    fn main()   {
        let mut st = "/dev/input/event6".to_string();
        let pt = to_c_string(&mut st);
        let fd = unsafe { libc::open(pt, libc::O_RDONLY | libc::O_NONBLOCK) };
    }

La protection de nos valeurs
============================

Pour terminer ce chapitre, nous allons améliorer le fonctionnement de l’appel à IOCTL. Pour rappel, voici comment il est codé pour l’instant.

.. code:: rust

    let mut ii = InputId    {
	    bustype : 0,
	    vendor  : 0,
	    product : 0,
	    version : 0
    };

    let io = unsafe {
        libc::ioctl(fd, 0x80084502 as c_ulong, &mut ii as *mut _)
    };

Il faut savoir que libdrm définit en son sein une version mieux conçue de ``ioctl``, implémentée comme suit.

.. code:: c

    int
    drmIoctl(int fd, unsigned long request, void *arg)
    {
        int ret;

        do {
        ret = ioctl(fd, request, arg);
        } while (ret == -1 && (errno == EINTR || errno == EAGAIN));
        return ret;
    }

Il peut arriver qu’un IOCTL échoue (donc renvoie ``-1``) sans qu’il y ait *réellement* eu d’erreur avec l’appel. Cela se produit dans deux cas.

- Lorsque le processus à reçu un signal (comme ``SIGINT``, ``SIGTERM``, ou d’autres moins nocifs comme ``SIGALRM``), qu’il a dû traiter, avant de reprendre son exécution normale. Cela met généralement la pagaille dans l’IOCTL, et celui-ci préfère laisser tomber avec l’erreur ``EINTR`` (pour *interruption*).
- Lorsque le périphérique est occupé et qu’il faut recommencer plus tard, l’erreur étant alors ``EAGAIN``.

Cette version améliorée de l’appel recommence donc l’IOCTL tant que l’une de ces deux situations est rencontrée, et n’abandonne que si une véritable erreur se produit.

Pour l’implémenter en Rust, il va nous falloir des boucles. Malheureusement, la boucle ``do``-``while`` n’existe pas en Rust. Voici donc un ersatz un peu ridicule, mais qui fait le travail.

.. code:: rust

    let mut ret : c_int;

    loop    {
        ret = unsafe { libc::ioctl(fd, 0x80084502 as c_ulong, &mut ii as *mut _) };
        if ret == -1 && (errno() == libc::EINTR || errno() == libc::EAGAIN)
             { continue; }
        else { break;    } // Ersatz moche de do-while.
    }

Les mots-clés ``break`` et ``continue`` devraient vous être familiers, je n’insiste pas. Quant à ``loop``, il crée une boucle infinie, en toute simplicité. Il existe également des boucles plus classiques, avec la syntaxe ``while <cond> { … }``, toujours sans parenthèses autour de la condition.

Il ne reste plus qu’à mettre tout cela, ainsi que la vérification du bon résultat, dans une fonction ``ioctl``. Aucun risque de collision avec la fonction d’origine, puisque celle-ci a besoin du préfixe ``libc::``. Mais vous allez assez vite rencontrer un problème : le type ``*mut _`` n’est pas accepté en dans la signature de type d’une fonction, il va impérativement falloir spécifier un type explicitement.

Utilisez donc ``*mut u8``, puisque C se fiche de savoir quel type avait le pointeur dans le code Rust qui l’appelle. Ce qui nous donne la fonction ci-dessous (ne regardez pas avant d’avoir essayé par vous-mêmes).

.. code:: rust

    fn ioctl(fd : c_int, request : u32, arg : *mut u8) -> c_int {
        let mut ret : c_int;

        loop    {
            ret = unsafe { libc::ioctl(fd, request as c_ulong, arg) };
            if ret == -1 && (errno() == libc::EINTR || errno() == libc::EAGAIN)
                 { continue; }
            else { break;    } // Ersatz moche de do-while.
        }

        if ret < 0   {
            panic!("L’IOCTL a échoué.");
        }

        ret
    }

Et l’appel dans ``main`` se résume à cette ligne.

.. code:: rust

    let _ = ioctl(fd, 0x80084502, &mut ii as *mut _ as *mut u8);

Convertir directement une référence vers un pointeur nu sur un autre type n’est pas autorisé : il faut passer par un pointeur nu non typé intermédiaire, car les conversions entre n’importe quels types de pointeurs nus sont autorisées, par contre. En outre, lier le résultat d’une fonction à ``_`` permet de s’en débarrasser sans que rustc ne râle.

Il reste une dernière chose à faire, et l’on pourra s’arrêter là : toutes les valeurs de requête ne donnent pas un IOCTL valide, ce serait plus sécurisé si la fonction n’acceptait que des requêtes valides en argument. Cela est possible en définissant une **énumération**, qui ne contiendra donc que les valeurs autorisées.

Pour l’instant, on va se contenter d’une énumération basique, qui suit la syntaxe suivante.

.. code:: rust

    enum <identifiant du type>  {
        <variante 1>,
        <variante 2>,
        <variante 3>,
        …
    }

L’identifiant du type et le nom des variantes doit être en *CamelCase*. Il est en outre possible de préciser une valeur numérique associée à une variante donnée, en la faisant suivre de ``= <nombre>``. Voici donc notre type ``IOCTL``, contenant une unique variante.

.. code:: rust

    enum IOCTL  {
        GetId = 0x80084502,
    }

On peut alors modifier le type de notre fonction ``ioctl`` pour qu’elle prenne un type ``IOCTL`` plutôt qu’un ``u32`` : on a alors la garantie que seules des valeurs autorisées pourront être utilisées comme requête. Ne reste plus qu’à modifier l’appel de fonction, comme suit.

.. code:: rust

    let _ = ioctl(fd, IOCTL::GetId, &mut ii as *mut _ as *mut u8);

En effet, dans une énumération, l’identifiant de type constitue un espace de noms auquel les identifiants des variantes sont rattachés : de cette manière, deux types peuvent avoir des variantes qui s’appellent pareil (un truc courant, comme ``None``, par exemple) sans qu’elles entrent en conflit. Si vous êtes vraiment sûrs de ce que vous faites, vous pouvez spécifier ``use <type>::*;`` pour exporter les variantes dans l’espace de noms général.

Compilez. C’est raté. Le compilateur nous envoie paître en nous expliquant que ``request`` a été déplacée par ``libc::ioctl`` dans une itération précédente de la boucle. Il n’y a rien à faire, aucun moyen de contourner avec des références : on est obligés de passer par le mécanisme dont on a parlé `plus haut`__ pour que la valeur soit copiée plutôt que déplacée.

.. __: copy_

Cela se fait en ajoutant la ligne ``#[derive(Clone, Copy)]`` avant la définition de notre énumération. Cette solution n’est pas toujours possible, et il est beaucoup trop tôt pour vous expliquer en quoi elle consiste *réellement*, mais pour l’instant, sachez qu’elle existe.

--------

Ce fut encore un long chapitre, avec de nouvelles notions à foison. N’hésitez pas à le relire à tête reposée, et à faire des essais de votre côté jusqu’à ce que vous ayez pleinement intégré le fonctionnement de la propriété et de l’emprunt, car c’est sûrement la notion la plus difficile à maîtriser en Rust, et elle est omniprésente.
