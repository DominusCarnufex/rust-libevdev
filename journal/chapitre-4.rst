À l’heure actuelle, notre code est moche. Il est encombré, lourd par endroits, il y en a des morceaux dans tous les sens sans réelle organisation : ce n’est pas ce que je vous avais vendu en vous parlant de Rust. C’est parce qu’il vous reste encore de nombreux outils du langage à découvrir. Ce chapitre sera similaire au `chapitre 2`__ : le code ne fera rien de plus à la fin qu’au début, mais il utilisera beaucoup plus d’outils afin d’être mieux écrit.

.. __: {{ site.baseurl }}/journal/chapitre-2.html

.. contents::

Discours de la méthode
======================

On a vu dans le chapitre 2 que les énumérations constituent un espace de noms dans lequel sont rangées les différentes variantes : toutes les variantes de l’énumération doivent être précédées de ``MonType::``, à moins de les importer dans l’espace de noms général au moyen de ``use MonType::*;``.

Tous les types pleins de Rust constituent un espace de noms, qu’il s’agisse d’énumérations, de structures, ou même des types natifs du langage : ce n’est pas le cas des références et pointeurs nus, en revanche. Et dans cet espace de noms, il est possible de définir des fonctions, à l’aide de la syntaxe ci-dessous.

.. code:: rust

    impl MonType    {

    // Les définitions de fonction vont ici.

    }

Pour appeler ces fonctions membres, il faut utiliser la syntaxe ``<type>::<fonction>``, qui doit commencer à vous être familière, à présent. Naturellement, n’importe quelle fonction peut être définie dans l’espace de noms d’un type ; cependant, il en existe deux sortes particulières, qui sont les plus couramment utilisées car les plus utiles.

La première sorte, ce sont les constructeurs personnalisés : ils renvoient un objet du type dont la fonction est membre, construit à partir des éventuels arguments de la fonction. Prenons un exemple, ce sera plus évident. Rappelez-vous cette fonction.

.. code:: rust

    fn new_input_id() -> InputId    {
        InputId {
            bustype : 0,
            vendor  : 0,
            product : 0,
            version : 0
        }
    }

Il serait beaucoup plus idiomatique de l’écrire ainsi.

.. code:: rust

    impl InputId    {
        fn new() -> Self    {
            InputId {
                bustype : 0,
                vendor  : 0,
                product : 0,
                version : 0
            }
        }
    }

Notez le type ``Self``, qui permet de remplacer l’identifiant du type que l’on est actuellement en train d’**implémenter**, et ce dans toutes les situations. Sauf quand on veut accéder aux variantes d’une énumération, curieusement. On peut alors modifier la création de l’``InputId`` dans la fonction ``main``.

.. code:: rust

    // let mut ii = new_input_id();
    let mut ii = InputId::new();

.. note::

    Les constructeurs personnalisés s’opposent aux constructeurs natifs, qui sont ceux fournis par la syntaxe elle-même : ``InputId { … }`` pour une structure, ``EventType::Synchro`` pour une énumération simple, ``0x42`` pour un type natif, etc.

La deuxième sorte, ce sont les fonctions qui prennent en premier (voire en seul) argument un objet du type implémenté, ou une référence vers ce type. Dans ce cas, dans la signature de type de la fonction membre, on remplace le premier argument par respectivement ``self``, ``&self`` et ``&mut self``. Là encore, un exemple sera plus parlant. Rappelez-vous l’appel à l’IOCTL qui suit la construction d’un ``InputId``.

.. code:: rust

    let _ = ioctl(fd, IOCTL::GetId, &mut ii as *mut _ as *mut u8);

La partie ``&mut ii as *mut _ as *mut u8`` pourrait être remplacée par une fonction qui prend pour seul argument une référence mutable sur un ``InputId``, donc par une fonction membre, que voici.

.. code:: rust

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self as *mut Self as *mut u8
    }

Trois choses sont à noter.

- Le nom ``as_mut_ptr`` est classique. Lorsqu’une fonction membre renvoie une référence vers l’objet concerné, ou plus généralement vers un élément constitutif mais caché de celui-ci, il est d’usage de l’appeler ``as_ref``. De même, ``as_mut`` renvoie une référence mutable, ``as_ptr`` renvoie un pointeur nu, et ``as_mut_ptr`` renvoie un pointeur nu mutable.
- Ce n’est pas forcément très intuitif, mais ``&mut self`` équivaut à ``self : &mut InputId``. Par conséquent, ``self`` est ici une référence mutable, mais dans une fonction de type ``fonction(self, …)``, ``self`` serait un ``InputId`` et non une référence. Attention donc au type exact du ``self`` que vous manipulez.
- À nouveau, on utilise ``Self`` à la place de l’identifiant du type.

L’intérêt principal de cette sorte de fonctions membres, c’est qu’on peut employer la **syntaxe de méthode**, c’est-à-dire la syntaxe ``<objet>.<fonction>`` que vous avez déjà rencontrée. Et ce qui est encore mieux, c’est que cette syntaxe se fiche de savoir si ``<objet>`` est un objet plein, une référence, ou même une référence de référence de référence : elle trouvera toujours la bonne fonction membre, et passera l’argument sous la bonne forme.

L’appel de l’IOCTL devient par conséquent ceci.

.. code:: rust

    let _ = ioctl(fd, IOCTL::GetId, ii.as_mut_ptr());

Ce qui est à la fois plus propre et plus clair quant à ce qu’on fait exactement.

Déconstructivisme
=================

Il y a un autre passage de la fonction ``main`` qui n’est vraiment pas propre, et qui mériterait qu’on l’améliore. C’est celui-ci.

.. code:: rust

    let mut st = "/dev/input/event6".to_string();
    let pt = to_c_string(&mut st);
    let fd = unsafe { libc::open(pt, libc::O_RDONLY | libc::O_NONBLOCK) };

    if fd < 0   {
        panic!("Impossible d’ouvrir le fichier {}.", st);
    }

Et avec la fonction appelée, pour avoir toutes les informations.

.. code:: rust

    fn to_c_string(st : &mut String) -> *const c_char   {
        st.push('\0');
        st.as_ptr() as *const c_char
    }

Pour améliorer ce code, nous allons définir un nouveau type, comme ceci.

.. code:: rust

    struct CString(String);

Ceci est une structure-tuple, et vous n’en avez encore jamais rencontré. Une structure-tuple est déclarée avec le mot-clé ``struct``, mais contrairement à une structure classique, ses champs ne sont pas nommés : on ne met donc entre parenthèses que les types de ces champs. Ici, il n’y en a qu’un, mais il peut y en avoir plusieurs, comme dans cet exemple.

.. code:: rust

    struct RGBA(u8, u8, u8, u8);

Quel est intérêt d’une telle structure, sachant qu’on ne disposera pas de champs nommés pour accéder aux données ? Il est assez spécifique, et c’est pour cela que vous ne rencontrerez pas beaucoup de structures-tuples. Il existe en Rust la possibilité de définir un synonyme de type, à l’aide du mot clé ``type``. Voici par exemple comment le type ``c_int`` est défini.

.. code:: rust

    type c_int = i32;

Les deux types sont véritablement synonymes : le compilateur ne fait aucune différence entre les deux, vous pouvez les employer l’un à la place de l’autre sans souci, c’est un pur confort d’écriture.

Seulement, il est interdit en Rust de définir des fonctions membres pour un type qui a été défini dans un autre *crate*. Vous ne connaissez pas encore le fonctionnement des *crates*, mais pour vous pour l’instant, cela signifie que seuls les types que *vous* avez définis peuvent être implémentés dans votre code. Et définir un synonyme ne fonctionnera pas, puisque le compilateur les traite comme deux types identiques.

C’est là qu’intervient la structure-tuple à un seul champ : il s’agit d’un type différent, créé par vous, vous pouvez donc l’implémenter. Mais le compilateur n’est pas idiot, il se rend bien compte que c’est juste un emballage autour du type de départ, et il optimise tout cela, si bien qu’il n’y a pas de perte de temps à l’exécution (*overhead*, en anglais).

Pour notre type ``CString``, voici les trois fonctions membres que l’on va créer.

.. code:: rust

    impl CString    {
        fn new(s : &str) -> Self    {
            let mut string = s.to_string();
            string.push('\0');
            CString(string)
        }

        fn as_ptr(&self) -> *const c_char   {
            let &CString(ref st) = self;
            st.as_ptr() as *const c_char
        }

        fn as_ref(&self) -> &str    {
            let &CString(ref st) = self;
            st
        }
    }

La fonction ``new`` ne devrait pas vous poser de problème, en revanche, vous devez vous demander ce qui se passe à la première ligne de chacune des deux autres fonctions. Il s’agit d’une liaison par **déconstruction**. Si vous avez l’habitude de la programmation fonctionnelle, vous devez être en terrain connu ; pour les autres, accrochez-vous !

On l’a vu, pour chaque type, il existe un constructeur natif fourni par la syntaxe de Rust. Lorsque ce type possède des champs, il est possible, lorsque l’on crée une liaison (à l’aide de ``let``), de lier directement le contenu de ces champs à des identifiants, plutôt que de lier l’objet global.

Ainsi, si l’on prend le type ``struct RGBA(u8, u8, u8, u8)`` de tout à l’heure, on peut tout d’abord **construire** un objet de ce type et le lier à un identifiant, selon une syntaxe que vous connaissez.

.. code:: rust

    let couleur = RGBA(0xff, 0x42, 0x00, 0x79);

Puis, plus tard dans le code, si on veut accéder aux champs de manière individuelle, on va pouvoir **déconstruire** cet objet, en utilisant le constructeur natif dans la partie gauche de la liaison, comme ceci.

.. code:: rust

    let RGBA(rouge, vert, bleu, trans) = couleur;
    println!("0x{:x} 0x{:x} 0x{:x} 0x{:x}", rouge, vert, bleu, trans);

C’est particulièrement utile, et cela fonctionne avec tous les constructeurs natifs, même si vous verrez très rarement le cas suivant.

.. code:: rust

    let InputId {
        bustype : bus,
        vendor  : vend,
        product : prod,
        version : vers
    } = ii;

    println!("La version est {}.", vers);

Notez que si certains champs ne vous intéressent pas, comme avec toutes les liaisons, vous pouvez les jeter avec l’eau du bain en remplaçant leur identifiant par ``_``, comme dans cet exemple.

.. code:: rust

    let RGBA(rouge, _, bleu, _) = couleur;
    println!("0x{:x} 0x{:x}", rouge, bleu);

Il y a cependant une difficulté : une liaison par déconstruction consume la donnée qui lui est fournie à droite, comme toutes les liaisons. Vous rencontrerez donc une erreur si vous essayez de déconstruire une référence, puisque vous allez essayer de prendre la propriété du contenu de ses champs, alors que vous n’avez pas la propriété de l’objet complet.

La solution, c’est de demander une référence vers la valeur contenue dans le champ, ce qui se fait à l’aide du mot-clé ``ref``, comme dans cet exemple.

.. code:: rust

    let couleur = RGBA(0xff, 0x42, 0x00, 0x79);
    let ref_couleur = &couleur;
    let RGBA(ref rouge, _, ref bleu, _) = *ref_couleur;
    println!("0x{:x} 0x{:x}", rouge, bleu);

Notez que lorsque vous déconstruisez une référence, vous pouvez au choix déréférencer le côté droit de la liaison (comme ici, avec ``*ref_couleur``), ou intégrer dans le côté gauche le fait qu’il s’agit d’une référence (ici, on aurait ``let &RGBA(…) = ref_couleur;``.

.. important::

    Il est très important que vous compreniez la différence entre ``&`` et ``ref``.

    - Si un objet ou un champ est *déjà* une référence lorsque vous lancez la déconstruction, vous le signalez grâce à ``&``.
    - Si cet objet *n’est pas* une référence, mais que vous voulez associer l’identifiant avec une référence vers cet objet, alors vous utilisez ``ref``.

À présent, revenons-en aux deux fonctions membres du ``CString``.

.. code:: rust

    fn as_ptr(&self) -> *const c_char   {
        let &CString(ref st) = self;
        st.as_ptr() as *const c_char
    }

    fn as_ref(&self) -> &str    {
        let &CString(ref st) = self;
        st
    }

Ici, ``self`` est une référence à un ``CString``, que l’on déconstruit, afin d’obtenir une référence vers le ``String`` qu’il contient, et associer cette référence à l’identifiant ``st``. On poursuit ensuite le traitement en utilisant ``st``.

Quant aux lignes qui se trouvaient dans la fonction ``main``…

.. code:: rust

    let mut st = "/dev/input/event6".to_string();
    let pt = to_c_string(&mut st);
    let fd = unsafe { libc::open(pt, libc::O_RDONLY | libc::O_NONBLOCK) };

    if fd < 0   {
        panic!("Impossible d’ouvrir le fichier {}.", st);
    }

… elles deviennent dès lors ceci.

.. code:: rust

    let name = CString::new("/dev/input/event6");
    let fd = unsafe {
        libc::open(name.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK)
    };

    if fd < 0   {
        panic!("Impossible d’ouvrir le fichier {}.", name.as_ref());
    }

Et la fonction ``to_c_string`` disparaît complètement.

.. note::

    La bibliothèque standard de Rust définit deux types ``CStr`` et ``CString`` qui ont une fonction proche du type défini ici, mais ils sont beaucoup plus difficiles à utiliser, alors tant pis pour eux.

Décor à motifs
==============

À la fin du chapitre 3, on avait codé un passage très moche permettant de créer un vecteur de ``EventType`` à partir d’un champ de bits. Un passage qui ressemblait à ceci.

.. code:: rust

    if (bits >> 0x01) % 0b10 == 1   {
        event_types.push(EventType::Key);
    }
    if (bits >> 0x02) % 0b10 == 1   {
        event_types.push(EventType::Relative);
    }
    if (bits >> 0x03) % 0b10 == 1   {
        event_types.push(EventType::Absolute);
    }

Je vous avais alors dit qu’on apprendrait dans ce chapitre un moyen d’écrire ce code de manière plus propre. C’est ce à quoi nous allons nous atteler, en commençant par définir un constructeur personnalisé pour le type ``EventType``. L’enveloppe de ce constructeur sera comme suit, ainsi que vous vous en doutez.

.. code:: rust

    impl EventType  {
        fn new(int : u8) -> Self    {
            // Le code ici.
        }
    }

Un ``u8`` suffit, puisque la valeur maximale possible est ``0x1f``. Mais à présent, avec les outils que vous connaissez, vous en êtes réduits à faire une grosse série de ``if``-``else if``-``else``. Il est donc temps d’introduire le **filtrage par motif**. Là encore, ceux qui ont un bagage en programmation fonctionnelle seront à l’aise, et les autres vont devoir s’accrocher.

Le filtrage par motif consiste à prendre une valeur donnée, et à essayer de la lier par déconstruction à une série de constructeurs différents appelés **motifs**. Lorsque l’un de ces motifs correspond à la valeur entrée, une certaine **branche** du filtrage par motif est exécutée. Voici à quoi cela ressemble.

.. code:: rust

    match <valeur>  {
        <motif 1> => <branche 1>,
        <motif 2> => <branche 2>,
        <motif 3> => <branche 3>,
        <motif 4> => <branche 4>,
        …
    }

Il y a cependant un certain nombre de règles.

- Les motifs doivent tous être des constructeurs du même type, et du même type que la valeur qui va être comparée aux motifs.
- Toutes les branches doivent renvoyer une valeur du même type. S’il y a plusieurs instructions dans une branche donnée, on les entoure d’accolades.
- La valeur est comparée au premier motif, puis au deuxième, puis au troisième, et ainsi de suite.
- Le filtrage doit être complet : tous les constructeurs d’un même type doivent être traités. Au besoin, le motif ``_`` permet de gérer tous les cas restants.

Dans le cas d’un ``u8``, les constructeurs sont ``0``, ``1``, etc. jusqu’à ``255``. Voici donc le code complet de notre constructeur de ``EventType``.

.. code:: rust

    impl EventType  {
        fn new(int : u8) -> Self    {
            match int   {
                0x00 => EventType::Synchro,
                0x01 => EventType::Key,
                0x02 => EventType::Relative,
                0x03 => EventType::Absolute,
                0x04 => EventType::Miscellaneous,
                0x05 => EventType::Switch,
                0x11 => EventType::LED,
                0x12 => EventType::Sound,
                0x14 => EventType::Repeat,
                0x15 => EventType::FF,
                0x16 => EventType::Power,
                0x17 => EventType::FFStatus,
                _    => panic!("EventType inconnu : 0x{:x}", int)
            }
        }
    }

Je le répète car c’est important : le filtrage par motifs fonctionne avec *tous* les constructeurs. Ainsi, cette fonction…

.. code:: rust

    fn as_ptr(&self) -> *const c_char   {
        let &CString(ref st) = self;
        st.as_ptr() as *const c_char
    }

… pourrait également s’écrire ainsi.

.. code:: rust

    fn as_ptr(&self) -> *const c_char   {
        match *self {
            CString(ref st) => st.as_ptr() as *const c_char
        }
    }

Revenons-en à notre fonction ``main``. Le code qui crée le vecteur de ``EventType`` est celui-ci, pour rappel.

.. code:: rust

    let mut event_types = Vec::new();

    event_types.push(EventType::Synchro); // Il doit nécessairement
                                          // être présent.
    if (bits >> 0x01) % 0b10 == 1   {
        event_types.push(EventType::Key);
    } 
    /* Les autres valeurs. */ 
    if (bits >> 0x17) % 0b10 == 1   {
        event_types.push(EventType::FFStatus);
    }

Avec le constructeur de ``EventType`` tout neuf, on pourrait remplacer ``EventType::Key`` et ``EventType::FFStatus`` par ``EventType::new(0x01)`` et ``EventType::new(0x17)`` respectivement, et pareil pour toutes les variables intermédiaires. Et là, votre âme de codeur C se dit que ce qui remplacerait élégamment cet amas de ``if``, ce serait une boucle ``for``.

Et vous avez raison. Il existe bien une boucle ``for`` en Rust, mais elle équivaut plutôt à une boucle ``foreach`` de Perl qu’à une boucle ``for`` de C. Voici sa syntaxe générale.

.. code:: rust

    for i in iterator   {

    }

La boucle va appliquer le même traitement à tous les éléments successifs d’un **itérateur**. On verra plus tard ce qu’est exactement un itérateur. Pour l’instant, vous allez juste découvrir le plus simple de tous les itérateurs : ``<n0>..<n>``. Les objets ``<n0>`` et ``<n>`` sont des nombres entiers, de n’importe quel type, et l’itérateur énumère tous les entiers de ``<n0>`` à celui qui précède ``<n>``.

Ainsi, ``0x01..0x20`` itère tous les entiers de ``0x01`` à ``0x1f``. Ce qui permet de réduire le code initial à ceci.

.. code:: rust

    let mut event_types = Vec::new();

    event_types.push(EventType::Synchro); // Il doit nécessairement
                                          // être présent.
    for i in 0x01..0x20 {
        if (bits >> i) % 0b10 == 1  {
            event_types.push(EventType::new(i));
        }
    }

Un gain plus qu’appréciable, donc ! C’est maintenant votre tour de coder. Vous allez réaliser la même opération d’assainissement sur toute la partie qui gère les codes d’événement. La solution est bien sûr juste en-dessous, pour quand vous aurez terminé de travailler.

Tout d’abord, le constructeur personnalisé.

.. code:: rust

    impl EventCode  {
        fn new(event_type : EventType, int : usize) -> Self {
            match event_type    {
                EventType::Key => match int {
                    0x110 => EventCode::ButtonLeft,
                    0x111 => EventCode::ButtonRight,
                    0x112 => EventCode::ButtonMiddle,
                    0x113 => EventCode::ButtonSide,
                    0x114 => EventCode::ButtonExtra,
                    0x115 => EventCode::ButtonForward,
                    0x116 => EventCode::ButtonBack,
                    0x117 => EventCode::ButtonTask,
                    _     => unimplemented!()
                },
                _ => unimplemented!()
            }
        }
    }

Et ensuite, le code dans la fonction ``main``, avec l’appel à l’IOCTL et l’affichage après coup, pour le contexte.

.. code:: rust

    let mut key_bits : [c_ulong; 12] = [0; 12];

    let _ = ioctl(fd, IOCTL::GetKeyBits, &mut key_bits as *mut _ as *mut u8);

    let mut event_codes = Vec::new();

    for i in 0x00..0x300    {
        let a = i / 64;
        if (key_bits[a] >> (i - 64 * a)) % 0b10 == 1    {
            event_codes.push(EventCode::new(EventType::Key, i));
        }
    }

    println!("libevdev_has_event_code(dev, EV_KEY, BTN_LEFT) = {}",
        event_codes.contains(&EventCode::ButtonLeft));

Notez que ``a`` étant utilisé comme indice d’un tableau, il est obligatoirement de type ``usize``, ce qui a pour conséquence que ``i`` est aussi un ``usize`` à cause de ``let a = i / 64;``. C’est pourquoi, par facilité, on a défini le constructeur de ``EventCode`` comme prenant un ``usize`` en argument.

----------

C’est ici que s’achève ce quatrième chapitre. Tâchez de bien comprendre comment fonctionne le filtrage par motifs avant de continuer. En effet, il vous apparaît sans doute pour l’instant comme un simple confort d’écriture : c’est parce que nous n’avons encore vu que les énumérations *simples*. Lorsque nous aurons vu les autres sortes d’énumérations, vous comprendrez pourquoi le filtrage par motifs est aussi utilisé en Rust.

Et parce qu’il fallait bien que cela arrive un jour, je vous laisse un exercice à réaliser avant de passer au chapitre 5. Votre mission : créer un type ``Device`` qui représente notre périphérique, avec un constructeur, et toutes les fonctions membres nécessaires pour laisser le moins de code possible dans ``main``, à l’exception des affichages.

La correction au prochain chapitre !
