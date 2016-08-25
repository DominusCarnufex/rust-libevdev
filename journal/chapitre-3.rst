Dans ce troisième chapitre, nous allons recommencer à ajouter de nouvelles fonctionnalités à notre programme, afin de nous rapprocher du code d’exemple en C qui nous sert de base. Pour cela, on limitera au maximum les nouvelles connaissances en Rust, afin de réellement nous concentrer sur notre mission.

.. contents::

Remise en forme
===============

Afin de se remettre en train pour la prochaine étape qui va demander pas mal d’exploration du code C, vous allez faire un petit exercice d’application. Votre mission : coder un équivalent Rust de la fonction ``libevdev_get_driver_version`` qui se trouve dans le fichier ``libevdev.c``.

La marche à suivre est quasiment identique à celle que nous avons employée dans le chapitre 1 pour les fonctions ``libevdev_get_id_*``. N’hésitez donc pas à vous en inspirer pour cet exercice. Quand vous aurez terminé, vous pourez regarder la correction succincte ci-dessous.

C’est bon ?

La fonction ``libevdev_get_driver_version`` se contente de renvoyer le champ ``driver_version`` de la grosse structure ``struct libevdev``. Dans le fichier ``libevdev-int.h``, on voit que ce champ est de type ``int``, et dans la fonction ``libevdev_set_fd``, ce champ est rempli grâce à l’IOCTL ``EVIOCGVERSION`` qui vaut ``0x80044501``.

Pour porter tout cela en Rust, il faut commencer par créer une nouvelle variante à l’énumération ``IOCTL``, que l’on appellera ``GetVersion`` et qui aura pour valeur ``0x80044501``. Ensuite, le code est quasiment identique à celui des trois fonctions déjà implémentées.

.. code:: rust

    let mut vers : libc::c_int = 0;

    let _ = ioctl(fd, IOCTL::GetVersion, &mut vers as *mut _ as *mut u8);

    println!("Version = 0x{:x}", vers);

À l’exécution (avec les droits d’administrateur, n’oubliez pas !), on obtient le résultat suivant.

.. code:: console

    Version = 0x10001

Un peu étrange comme numéro de version, jusqu’à ce que l’on comprenne qu’il s’agit en fait de ``01.00.01``, chaque octet contenant un des éléments de la version, selon le modèle ``majeure.mineure.patch``. On va donc afficher cela sous une forme plus lisible.

.. code:: rust

    println!("Version = {}.{}.{}", (vers >> 16) % 0x100, (vers >> 8) % 0x100, vers % 0x100);

Comme vous le voyez, les opérateurs mathématiques et de manipulation bit-à-bit sont les mêmes qu’en C.

Comprendre
==========

On va maintenant passer aux choses sérieuses, et tenter d’implémenter en Rust les fonctions ``libevdev_has_event_type`` et ``libevdev_has_event_code``. Pour cela, il faut déjà comprendre comment elles fonctionnent, et elles sont sensiblement plus complexes que les fonctions avec lesquelles on a travaillé pour l’instant. Alors commençons par leur code source, qui se trouve dans ``libevdev.c``.

.. code:: c

    LIBEVDEV_EXPORT int
    libevdev_has_event_type(const struct libevdev *dev, unsigned int type)
    {
        return type == EV_SYN ||(type <= EV_MAX && bit_is_set(dev->bits, type));
    }

    LIBEVDEV_EXPORT int
    libevdev_has_event_code(const struct libevdev *dev, unsigned int type, unsigned int code)
    {
        const unsigned long *mask = NULL;
        int max;

        if (!libevdev_has_event_type(dev, type))
            return 0;

        if (type == EV_SYN)
            return 1;

        max = type_to_mask_const(dev, type, &mask);

        if (max == -1 || code > (unsigned int)max)
            return 0;

        return bit_is_set(mask, code);
    }

Intéressons-nous d’abord à ``libevdev_has_event_type`` qui est visiblement plus simple. Elle prend comme argument le pointeur sur la ``struct libevdev``, rien d’étonnant là-dedans, et un ``unsigned int`` représentant le type d’événements dont on veut savoir si le périphérique peut en envoyer.

Si le type d’événements est ``EV_SYN``, la fonction renvoie « vrai ». Sinon, elle vérifie que le type d’événements soit inférieur à une valeur ``EV_MAX``, puis appelle une fonction ``bit_is_set`` sur le champ ``bits`` de la ``struct libevdev`` et le type d’événements. Partons donc à la recherche de ces différents éléments. Dans ``include/linux/input-event-codes.h``, on trouve ceci.

.. code:: c

    /*
     * Event types
     */

    #define EV_SYN          0x00
    #define EV_KEY          0x01
    #define EV_REL          0x02
    #define EV_ABS          0x03
    #define EV_MSC          0x04
    #define EV_SW           0x05
    #define EV_LED          0x11
    #define EV_SND          0x12
    #define EV_REP          0x14
    #define EV_FF           0x15
    #define EV_PWR          0x16
    #define EV_FF_STATUS    0x17
    #define EV_MAX          0x1f
    #define EV_CNT          (EV_MAX+1)

Nous avons donc la liste complète des types d’événements, ainsi que la valeur de ``EV_MAX`` qui, sans surprise, représente la valeur maximale autorisée pour un type d’événements. Quant à ``bit_is_set``, elle est définie dans ``libevdev-util.h``.

.. code:: c

    static inline int
    bit_is_set(const unsigned long *array, int bit)
    {
        return !!(array[bit / LONG_BITS] & (1LL << (bit % LONG_BITS)));
    }

Sur le coup, ce n’est vraiment pas clair, alors allons chercher la définition du champ ``bits``, que voici.

.. code:: c

    unsigned long bits[NLONGS(EV_CNT)];

On retrouve ``EV_CNT`` que nous avons découvert quelques lignes plus haut, et l’on peut compléter avec les lignes suivantes tirées de ``libevdev-util.h``.

.. code:: c

    #define LONG_BITS (sizeof(long) * 8)
    #define NLONGS(x) (((x) + LONG_BITS - 1) / LONG_BITS)

Avec toutes ces informations en main, à votre avis, que se passe-t-il dans tout ce bazar ? Réfléchissez-y un peu, puis continuez à lire.

En C, il n’existe pas à proprement parler de type booléen : une valeur de 0 dans un type numérique ou de pointeur vaut « faux », et toute autre valeur vaut « vrai ». Par conséquent, si on a 32 valeurs possibles de type d’événements, utiliser ne serait-ce qu’un ``char`` pour chaque type d’événements prend 32 octets en mémoire. Tandis qu’une représentation sous forme de champ de bits (*bitfield*) permet de stocker la même information dans seulement 4 octets.

C’est ce qui se passe ici : le champ ``bits`` est un tableau de ``unsigned long`` comportant le nombre de ``unsigned long`` nécessaires pour représenter ``EV_CNT`` bits. Et ``bit_is_set`` vérifie si le ``i``-ème bit vaut 0 ou 1.

Pourquoi une définition si compliquée du champ ``bits`` ? Parce que si vous regardez la définition de la ``struct libevdev``, vous verrez que les champs ``key_bits``, ``rel_bits``, ``abs_bits``, etc. (un pour chaque type d’événements, en gros) fonctionnent sur le même modèle.

Et autant, avec un maximum de 32 types d’événements, on est certain que cela tienne dans un seul ``unsigned long`` (qui fait au mininum 32 bits, d’après le standard C), autant d’autres champs peuvent avoir plus de valeurs possibles : en l’occurrence, si vous parcourez ``include/linux/input-event-codes.h``, vous verrez que cela ne concerne que ``key_bits`` et ``abs_bits``.

Revenons à ``libevdev_has_event_code``. La structure générale est la même, à ceci près qu’elle prend un code d’événement en plus du type d’événements. Si le périphérique ne possède pas le type d’événements demandé, la fonction renvoie « faux », si le type demandé est ``EV_SYN``, elle renvoie « vrai » quel que soit le code d’événement. Vient la ligne suivante, que l’on expliquera un peu plus loin.

.. code:: c

    max = type_to_mask_const(dev, type, &mask);

Si cette fonction a échoué (``max`` vaut ``-1``) ou si le code demandé est supérieur à ``max``, la fonction renvoie « faux ». Sinon, elle utilise ``bit_is_set`` sur ``mask`` et le code demandé, de la même manière que ce qu’on a vu plus haut.

Alors que fait-elle cette fonction ``type_to_mask_const`` ? Elle est définie comme suit dans ``libevdev-int.h``, avec la macro nécessaire à sa bonne compréhension en bonus.

.. code:: c

    #define max_mask(uc, lc) \
        case EV_##uc: \
                *mask = dev->lc##_bits; \
                max = libevdev_event_type_get_max(type); \
                break;

    static inline int
    type_to_mask_const(const struct libevdev *dev, unsigned int type, const unsigned long **mask)
    {
        int max;

        switch(type) {
            max_mask(ABS, abs);
            max_mask(REL, rel);
            max_mask(KEY, key);
            max_mask(LED, led);
            max_mask(MSC, msc);
            max_mask(SW, sw);
            max_mask(FF, ff);
            max_mask(REP, rep);
            max_mask(SND, snd);
            default:
                 max = -1;
                 break;
        }

        return max;
    }

En résumé, pour un type d’événements ``EV_ABS``, mask renvoie le champ ``abs_bits`` de la ``struct libevdev``, pour ``EV_REL``, c’est ``rel_bits``, etc. Quant à ``max``, c’est la fonction ``libevdev_event_type_get_max`` qui le définit, et cette fonction se trouve dans ``libevdev.c``.

.. code:: c

    LIBEVDEV_EXPORT int
    libevdev_event_type_get_max(unsigned int type)
    {
        if (type > EV_MAX)
            return -1;

        return ev_max[type];
    }

Après une nouvelle vérification que le type d’événements ne dépasse pas le maximum autorisé, on renvoie le champ correspondant au type dans ``ev_max``. Je vous épargne cette dernière étape, car cette variable se trouve dans un code source qui est généré par un script Python à la compilation de libevdev : c’est juste un tableau contenant toutes les valeurs ``*_MAX`` de ``include/linux/input-event-codes.h``.

Pfiou ! On est enfin arrivé au bout. Tracer le fonctionnement d’une fonction est souvent un véritable jeu de piste comme celui que nous venons de faire. Et il reste un dernier petit bout de chemin à faire : comment les champs ``*_bits`` sont-ils remplis en premier lieu ?

C’est naturellement du côté de ``libevdev_set_fd`` qu’il faut se tourner, et on y trouve les lignes suivantes.

.. code:: c

    rc = ioctl(fd, EVIOCGBIT(0, sizeof(dev->bits)), dev->bits);
    if (rc < 0)
        goto out;

    // Pour la liste de types d’événements.

    rc = ioctl(fd, EVIOCGBIT(EV_REL, sizeof(dev->rel_bits)), dev->rel_bits);
    if (rc < 0)
        goto out;

    rc = ioctl(fd, EVIOCGBIT(EV_ABS, sizeof(dev->abs_bits)), dev->abs_bits);
    if (rc < 0)
        goto out;

    rc = ioctl(fd, EVIOCGBIT(EV_LED, sizeof(dev->led_bits)), dev->led_bits);
    if (rc < 0)
        goto out;

    rc = ioctl(fd, EVIOCGBIT(EV_KEY, sizeof(dev->key_bits)), dev->key_bits);
    if (rc < 0)
        goto out;

    rc = ioctl(fd, EVIOCGBIT(EV_SW, sizeof(dev->sw_bits)), dev->sw_bits);
    if (rc < 0)
        goto out;

    rc = ioctl(fd, EVIOCGBIT(EV_MSC, sizeof(dev->msc_bits)), dev->msc_bits);
    if (rc < 0)
        goto out;

    rc = ioctl(fd, EVIOCGBIT(EV_FF, sizeof(dev->ff_bits)), dev->ff_bits);
    if (rc < 0)
        goto out;

    rc = ioctl(fd, EVIOCGBIT(EV_SND, sizeof(dev->snd_bits)), dev->snd_bits);
    if (rc < 0)
        goto out;

On a là un cas d’IOCTL un peu plus complexe que la dernière fois. En effet, c’est toujours la même macro ``EVIOCGBIT`` qui est utilisée à tous les appels, mais cette fois, elle prend des paramètres. Voici comment elle est définie dans ``include/linux/input.h``.

.. code:: c

    #define EVIOCGBIT(ev,len)    _IOC(_IOC_READ, 'E', 0x20 + (ev), len)

On a donc une base ``0x20`` à laquelle s’ajoute le code du type d’événements, et la longueur du tableau de ``unsigned long`` est prise en compte aussi. On va donc ressortir du placard notre code C servant à trouver la valeur des IOCTL.

.. code:: c

    #include <stdio.h>
    #include <sys/ioctl.h>

    #define EVIOCGBIT(ev,len)    _IOC(_IOC_READ, 'E', 0x20 + (ev), len)

    int main()  {
        printf("EVIOCGBIT(0, 0) = 0x%x\n", EVIOCGBIT(0, 0));
        printf("EVIOCGBIT(1, 0) = 0x%x\n", EVIOCGBIT(1, 0)); // EV_KEY
        printf("EVIOCGBIT(0, 4) = 0x%x\n", EVIOCGBIT(0, 4)); // long = 32 bits
        printf("EVIOCGBIT(0, 8) = 0x%x\n", EVIOCGBIT(0, 8)); // long = 64 bits

        return 0;
    }

Ce qui nous donne le résultat suivant.

.. code:: console

    EVIOCGBIT(0, 0) = 0x80004520
    EVIOCGBIT(1, 0) = 0x80004521
    EVIOCGBIT(0, 4) = 0x80044520
    EVIOCGBIT(0, 8) = 0x80084520

Ce qui nous permet donc de dire que l’IOCTL pour obtenir la liste des types d’événements disponibles est ``0x80044520`` si les ``long`` font 4 octets de long sur votre plateforme, et ``0x80084520`` s’ils en font 8. Quant à ``key_bits``, la valeur de ``KEY_CNT`` est ``0x300``, ce qui est un multiple de 8 : il faudra donc toujours ``0x60`` octets, quelle que soit la longueur des ``long``. Soit un IOCTL de ``0x80604521``.

.. note::

    Sur ma machine, les ``long`` font 64 bits. J’utiliserai donc désormais les valeurs correspondantes, à vous d’adapter si vos ``long`` font 32 bits.

Imiter
======

Il est maintenant temps d’adapter tout cela aux outils que nous offre Rust. La première chose à faire est naturellement d’ajouter deux variantes ``GetBits`` et ``GetKeyBits`` au type ``IOCTL``. Ensuite, on va commencer par adapter ``libevdev_has_event_type``, puisqu’elle est plus simple, et que l’autre fonction s’appuie dessus.

Une chose apparaît clairement : le besoin de vérifier que l’argument ``type`` est en-dessous d’un certain maximum naît du fait que c’est un ``unsigned int`` qui est passé à la fonction. Avec un type dédié comme ce que l’on a fait pour l’IOCTL, on a la garantie que seules les valeurs légales pourront être utilisées.

Créons donc un type ``EventType`` servant à représenter un type d’événements. Pour rappel, voici la liste définie dans ``include/linux/input-event-codes.h``.

.. code:: c

    #define EV_SYN          0x00
    #define EV_KEY          0x01
    #define EV_REL          0x02
    #define EV_ABS          0x03
    #define EV_MSC          0x04
    #define EV_SW           0x05
    #define EV_LED          0x11
    #define EV_SND          0x12
    #define EV_REP          0x14
    #define EV_FF           0x15
    #define EV_PWR          0x16
    #define EV_FF_STATUS    0x17

Objectivement, les noms ne sont pas très parlants. En parcourant les commentaires délimitant des sections dans ``include/linux/input-event-codes.h``, on trouve la signification de la plupart de ces abréviations. Voici donc le type que l’on va définir en Rust, avec la ligne ``#[derive(Clone, Copy)]`` pour ne pas s’embêter avec les histoires de propriété.

.. code:: rust

    #[derive(Clone, Copy)]
    enum EventType  {
        Synchro       = 0x00,
        Key           = 0x01,
        Relative      = 0x02,
        Absolute      = 0x03,
        Miscellaneous = 0x04,
        Switch        = 0x05,
        LED           = 0x11,
        Sound         = 0x12,
        Repeat        = 0x14,
        FF            = 0x15, // Pas trouvé à quoi cela correspond.
        Power         = 0x16,
        FFStatus      = 0x17 // Idem.
    }

Passons à présent à l’IOCTL. Le code est très similaire à ce que l’on a toujours fait.

.. code:: rust

    let mut bits : libc::c_long = 0;

    let _ = ioctl(fd, IOCTL::GetBits, &mut bits as *mut _ as *mut u8);

    println!("Bitfield = 0b{:b}", bits);

Notez le modificateur ``:b`` dans ``println``, pour afficher un nombre en notation binaire, plutôt qu’hexadécimale. Voici le résultat obtenu chez moi pour ma souris (il peut naturellement être différent chez vous).

.. code:: console

    Bitfield = 0x10111

Cela signifie que ma souris possède les types d’événements ``Synchro``, ``Key``, ``Relative`` et ``Miscellaneous``.

Maintenant, est-ce une bonne manière de conserver en mémoire la liste des types d’événements du périphérique, quand on code en Rust ? Non. C’est indubitablement le plus efficace en termes d’occupation mémoire, mais pas en termes de clarté du code ni de facilité d’utilisation.

À la place, on va utiliser un vecteur. Pour ceux qui ne seraient pas familiers des conteneurs usuels de C++ et autres langages orientés objet, un vecteur est un tableau dont on ne connaît pas la taille au moment de la compilation, et dont le nombre d’éléments peut évoluer au cours du temps.

En Rust, on crée un vecteur vide à l’aide de la fonction ``Vec::new()``. Ne posez pas encore de question sur la syntaxe utilisée, on la verra dans le prochain chapitre. Ensuite de quoi, on lui ajoute des éléments à l’aide de la méthode ``push(élément)``. Il est impératif que tous les éléments soient du même type, et que le vecteur soit mutable.

Là où le vecteur va s’avérer très intéressant pour nous, c’est qu’il dispose d’une méthode ``contains(&valeur)``, qui vérifie si le vecteur contient la valeur passée en argument.

.. note::

    Cette fonction renvoie un type ``bool``, qui est un type natif de Rust, et peut prendre les valeurs ``true`` et ``false``.

Ne reste plus qu’à remplir un vecteur de ``EventType``, en n’y mettant que ceux dont dispose le périphérique. La principale difficulté va être de convertir une valeur entière (le type tel que C le représente) en une valeur de type ``EventType``. Voici donc le code complet.

.. code:: rust

    let mut event_types = Vec::new();

    event_types.push(EventType::Synchro); // Il doit nécessairement
                                          // être présent.
    if (bits >> 0x01) % 0b10 == 1   {
        event_types.push(EventType::Key);
    } 
    if (bits >> 0x02) % 0b10 == 1   {
        event_types.push(EventType::Relative);
    } 
    if (bits >> 0x03) % 0b10 == 1   {
        event_types.push(EventType::Absolute);
    } 
    if (bits >> 0x04) % 0b10 == 1   {
        event_types.push(EventType::Miscellaneous);
    } 
    if (bits >> 0x05) % 0b10 == 1   {
        event_types.push(EventType::Switch);
    } 
    if (bits >> 0x11) % 0b10 == 1   {
        event_types.push(EventType::LED);
    } 
    if (bits >> 0x12) % 0b10 == 1   {
        event_types.push(EventType::Sound);
    } 
    if (bits >> 0x14) % 0b10 == 1   {
        event_types.push(EventType::Repeat);
    } 
    if (bits >> 0x15) % 0b10 == 1   {
        event_types.push(EventType::FF);
    } 
    if (bits >> 0x16) % 0b10 == 1   {
        event_types.push(EventType::Power);
    } 
    if (bits >> 0x17) % 0b10 == 1   {
        event_types.push(EventType::FFStatus);
    } 

    println!("libevdev_has_event_type(dev, EV_REL) = {}",
        event_types.contains(&EventType::Relative));
    println!("libevdev_has_event_type(dev, EV_KEY) = {}",
        event_types.contains(&EventType::Key));

C’est très laid, on en conviendra. Il est possible de faire cela de manière beaucoup plus élégante, mais là encore, cela attendra le prochain chapitre : on cherche ici à imiter les fonctions C, avec le moins possible d’outils nouveaux.

Mais surtout, cela ne compile pas. Le message d’erreur ne vous parlera certainement pas, et il est trop tôt encore pour l’expliquer. La version simple, c’est que le type ``EventType`` n’est pas prévu pour fonctionner avec l’opérateur ``==``, lequel est indispensable au fonctionnement de la méthode ``contains``. On va prendre un chemin de contournement : ôtez la ligne juste au-dessus de la définition du type ``EventType``, et remplacez-la par celle-ci.

.. code:: rust

    #[derive(Clone, Copy, PartialEq)]

Maintenant, le code compile, et le résultat est à la hauteur des attentes. Par ailleurs, souvenez-vous du code C qui implémente la fonction.

.. code:: c

    LIBEVDEV_EXPORT int
    libevdev_has_event_type(const struct libevdev *dev, unsigned int type)
    {
        return type == EV_SYN ||(type <= EV_MAX && bit_is_set(dev->bits, type));
    }

Non seulement on s’est débarrassé comme prévu de la partie ``type <= EV_MAX``, mais aussi de la partie ``type == EV_SYN`` : celui-ci se trouve dans le vecteur, et nécessairement en première position, donc inutile d’avoir un cas particulier pour lui, et la vérification sera presque aussi rapide qu’avec un simple ``type == EventType::Synchro``. Le corps de la fonction se résume en Rust à ceci.

.. code:: rust

    event_types.contains(&my_type)

Un beau résultat, n’est-ce pas ? Alors essayons de le reproduire pour ``libevdev_has_event_code``. Afin de simplifier les choses, on va se concentrer sur le cas de ``libevdev_has_event_code(EV_KEY, …)``, et comme ``EV_KEY`` a plus de 500 codes d’événement, on va là aussi se restreindre aux codes qui nous intéressent dans le cas d’une souris. Les voici, tout droit sortis de ``include/linux/input-event-codes.h``.

.. code:: c

    #define BTN_MOUSE       0x110
    #define BTN_LEFT        0x110
    #define BTN_RIGHT       0x111
    #define BTN_MIDDLE      0x112
    #define BTN_SIDE        0x113
    #define BTN_EXTRA       0x114
    #define BTN_FORWARD     0x115
    #define BTN_BACK        0x116
    #define BTN_TASK        0x117

Notez que ``BTN_MOUSE`` et ``BTN_LEFT`` sont des synonymes,  on ne gardera donc que le second. Et voici en toute logique notre type Rust correspondant.

.. code:: rust

    #[derive(Clone, Copy, PartialEq)]
    enum EventCode  {
        ButtonLeft    = 0x110,
        ButtonRight   = 0x111,
        ButtonMiddle  = 0x112,
        ButtonSide    = 0x113,
        ButtonExtra   = 0x114,
        ButtonForward = 0x115,
        ButtonBack    = 0x116,
        ButtonTask    = 0x117,
    }

Il nous faut à présent appeler l’IOCTL ``GetKeyBits`` que nous avons déjà défini, mais que lui passer en argument ? Avec ``0x300`` valeurs possibles, aucun type connu jusqu’à présent ne peut stocker autant de bits. On va faire comme en C : utiliser un tableau. Contrairement au vecteur, le tableau doit avoir une longueur connue à la compilation, et celle-ci ne changera pas. En outre, on est certain que les éléments du tableau seront contigus en mémoire.

Un tableau de ``N`` éléments de type ``T`` a pour type ``[T; N]``. Et pour initialiser tous ses éléments à une même valeur ``V``, la syntaxe est en fait la même : ``[V; N]``. Pour des valeurs différentes, la syntaxe est ``[V1, V2, V3, …]``. Enfin, pour accéder au ``n``-ième élément du tableau, la syntaxe est ``tab[n]``, sans grande surprise (les indices commençant à 0, comme en C).

Maintenant, c’est à vous. Écrivez tout le morceau de code qui récupère les ``key_bits`` du périphérique et stocke ceux qui correspondent à un bouton de souris dans un vecteur, avant de vérifier si ``BTN_LEFT`` fait partie du lot. La solution est juste en dessous, mais ne regardez pas tout de suite.

.. code:: rust

    let mut key_bits : [c_ulong; 12] = [0; 12];

    let _ = ioctl(fd, IOCTL::GetKeyBits, &mut key_bits as *mut _ as *mut u8);

    let mut event_codes = Vec::new();

    if (key_bits[4] >> (0x110 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonLeft);
    }
    if (key_bits[4] >> (0x111 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonRight);
    }
    if (key_bits[4] >> (0x112 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonMiddle);
    }
    if (key_bits[4] >> (0x113 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonSide);
    }
    if (key_bits[4] >> (0x114 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonExtra);
    }
    if (key_bits[4] >> (0x115 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonForward);
    }
    if (key_bits[4] >> (0x116 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonBack);
    }
    if (key_bits[4] >> (0x117 - 64 * 4)) % 0b10 == 1    {
        event_codes.push(EventCode::ButtonTask);
    }

    println!("libevdev_has_event_code(dev, EV_KEY, BTN_LEFT) = {}",
        event_codes.contains(&EventCode::ButtonLeft));

Encore une fois, le code qui sert à créer le vecteur est très laid, mais on verra comment améliorer cela dans le prochain chapitre. Et pour le reste, le code de la fonction ``libevdev_has_event_code`` elle-même est drastiquement simplifié. Pour rappel, voici l’original en C (sans les fonctions annexes qui rendent le tout encore plus complexe).

.. code:: c

    LIBEVDEV_EXPORT int
    libevdev_has_event_code(const struct libevdev *dev, unsigned int type, unsigned int code)
    {
        const unsigned long *mask = NULL;
        int max;

        if (!libevdev_has_event_type(dev, type))
            return 0;

        if (type == EV_SYN)
            return 1;

        max = type_to_mask_const(dev, type, &mask);

        if (max == -1 || code > (unsigned int)max)
            return 0;

        return bit_is_set(mask, code);
    }

Le code C a besoin de connaître le type d’événements, car un code d’événement de 0 voudra dire ``REL_X`` si le type est ``EV_REL``, mais ``SW_LID`` si le type est ``EV_SW``. *A contrario*, en Rust, on a un unique type ``EventCode``, et ``RelativeX`` et ``SwitchLid`` (lorsqu’elles existeront, bien sûr) sont des valeurs distinctes, même si elles donnent la même valeur numérique quand on les convertit en un type entier.

Plus besoin donc de vérifier que le périphérique autorise le type d’événements concerné. Plus besoin non plus de vérifier que le code d’événement est en-dessous d’une certaine limite. Et comme on a un unique vecteur de ``EventCode``, plus besoin non plus d’une fonction annexe pour déterminer dans quel champ aller faire les vérifications.

Et là encore, le cœur de notre fonction se réduit à une simple ligne.

.. code:: rust

    event_codes.contains(&code)

Un gain indéniable !

----------

C’est tout pour ce chapitre. Très peu de notions nouvelles en ce qui concerne Rust : les booléens, les vecteurs et les tableaux seulement. En revanche, vous avez pu vous rendre compte de l’important travail d’exploration du code d’origine qui accompagne toute transposition dans un autre langage. Vous avez également pu commencer à découvrir comment les outils de Rust permettent d’écrire un code qui se distingue nettement de l’original en C.
