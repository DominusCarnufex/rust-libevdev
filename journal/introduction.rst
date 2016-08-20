Ce « journal de développement » est organisé comme un cours d’introduction à Rust pour ceux qui s’intéressent à la programmation système et savent déjà programmer. Naturellement, il ne présuppose aucune connaissance préalable de Rust de votre part, et vous fera découvrir ce langage pas à pas.

En revanche, il est attendu que vous soyez familiers de C. En programmation système, il est absolument impossible d’échapper à ce langage, et le module evdev avec lequel nous cherchons à communiquer est lui-même écrit en C. Pas besoin d’être un gourou du langage, mais au minimum de le connaître suffisamment pour pouvoir lire un code C et comprendre ce qu’il fait, en se reportant si besoin à de la documentation.

En outre, s’il n’est pas nécessaire non plus d’être un expert en programmation système, attendez-vous à rencontrer quelques passages ardus si vous n’y connaissez *vraiment* rien.

.. question::

    Pourquoi libevdev en particulier ?

Premièrement, parce qu’on ne peut pas faire plus proche du noyau Linux sans réécrire une partie dudit noyau. On est ainsi presque au cœur de la programmation système. De plus, notre bibliothèque pourra être réellement *utile*, puisque la plupart des programmes y recourent directement ou indirectement.

Deuxièmement, parce que c’est un choix abordable. La quasi-totalité de son code source tient dans 2500 lignes de code C, ce qui est peu pour une bibliothèque aussi utile. Pour prendre un point de comparaison, il existe un autre module, appelé *direct rendering manager*, qui gère la communication directe avec les écrans, et qui est doté de sa propre bibliothèque assistante, libdrm. Les seules fonctions basiques de celle-ci représentent plus de 4700 lignes de code.

Troisièmement, parce qu’elle est assez simple d’utilisation. Il n’est pas nécessaire d’acquérir une demi-douzaine de concepts nouveaux avant de pouvoir simplement comprendre comment elle est organisée, contrairement à — au hasard — libdrm. Ce qui a pour conséquence qu’il est assez vite possible d’obtenir des résultats significatifs, même s’ils sont naturellement moins impressionnants qu’avec libdrm.

Quatrièmement, parce qu’elle n’est pas terminée. En effet, toutes les possibilités offertes par evdev ne sont pas couvertes par libevdev, et les développeurs sont parfois obligés de communiquer directement avec evdev, sans filet. Cela va donc nous obliger tôt ou tard à plonger dans le code du noyau lui-même, ce qui est toujours une expérience intéressante quand on s’intéresse à la programmation système.

Sur ce, bonne lecture !

{% include liste-chapitres.html %}

Pour finir, voici quelques liens qui pourront vous être utiles en permanence si vous vous décidez à apprendre Rust.

- Le `cours officiel`__ d’apprentissage de Rust. Si vous trouvez que je suis allé trop vite sur une notion, n’hésitez pas à vous y reporter pour avoir des informations complémentaires.
- La `documentation de la bibliothèque standard`__, toujours utile.
- La `documentation du paquet *libc*`__, dans sa version pour Linux.

.. __: https://doc.rust-lang.org/book/
.. __: https://doc.rust-lang.org/std/
.. __: https://doc.rust-lang.org/libc/x86_64-unknown-linux-gnu/libc/
