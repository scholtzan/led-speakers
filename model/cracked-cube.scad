$fn = 30;

module shell() {
    translate([-5, -5, -5]) {
        difference() {
            cube([10, 10, 10]);
            translate([0.5, 0.5, 0.5]) {
                cube([9, 9, 9]);
            }
        }
    }
}

module random_planes() {
    for (i = [0 : 15] ){
        translate([rands(-4, 4, 1)[0], rands(-4, 4, 1)[0], rands(-4, 4, 1)[0]]) {
            rotate([rands(20, 160, 1)[0], rands(20, 160, 1)[0], rands(20, 160, 1)[0]]) {
                translate([-25, -25, 0]) {
                    cube([50, 50, 0.3]);
                }
            }
        }
    }
}

module random_circles() {
    for (i = [0: 25]) {
        translate([rands(-8, 8, 1)[0], rands(-8, 8, 1)[0], -20]) {
            rotate([0, 0, rands(0, 180, 1)[0]]) {
                difference(){
                    s = rands(1, 6, 1)[0];
                    cylinder(50, 0.5 * s, 0.5 * s);
                    cylinder(50, 0.45 * s, 0.45 * s);
                }
            }
        }
    }
}

difference() {
    shell();
    random_circles();
    //random_planes();
}

