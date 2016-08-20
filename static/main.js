serif = false;

function toggle_serif() {
    if (serif)  {
        document.body.className = "sans";
        document.getElementById("btnserif").innerHTML = "Police à empattements";
        serif = false;
    } else {
        document.body.className = "serif";
        document.getElementById("btnserif").innerHTML = "Police sans empattement";
        serif = true;
    }
}
