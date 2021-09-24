// Speaker front cover

include <parameters.scad>;

module coverSphere() {
    difference() {
        sphere(frontRectLength * 5);
        sphere(frontRectLength * 5 - coverThickness * 2);
    }
}

module speakerCover() {
    $fn = 100;
    
    difference() {
        intersection() {
            coverSphere();

            translate([-frontRectLength / 2, -frontRectLength / 2, frontRectLength * 5 - frontRectLength / 2]) {
                cube([frontRectLength, frontRectLength, coverThickness * 10]);
            }
        }
        
        cylinder(r=speakerConeDiameter/2 + 3, h=frontRectLength * 5);
        
        // screw mounting holes
        for (i = [-1 : 2 : 1]) {
            for (j = [-1 : 2 : 1]) {
                translate([
                    wallLength / 2 * i - screwCenterDistance * i, 
                    wallLength / 2 * j - screwCenterDistance * j,
                    0
                ]) {
                    linear_extrude(height = frontRectLength * 5 - 10) {
                        circle(d=screwDiameter * 1.5);
                    }
                }
            }
        }
    }
}

//speakerCover();
