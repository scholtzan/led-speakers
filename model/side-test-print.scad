$fa = 0.1;
$fs = 0.4;


spacing = 0.2 ;    // degrees
spacingAbs = 0.5; // mm
shortSide = 0.9; // degrees
arms = 0.4; // degrees
length = 20;
wallLength = 160;
wallThickness = 0.3;
hexHeight = 0.3;


module sideWall(width, height, thickness) {
    color("black") {
        cube([width, height, thickness]);
    }
}

module hexHalf(length, px0, px1, px2, px3, px4, px5, space, mirrored) {
    x0 = mirrored ? px5 : px0;
    x1 = mirrored ? px4 : px1;
    x2 = mirrored ? px3 : px2;
    x3 = mirrored ? px2 : px3;
    x4 = mirrored ? px1 : px4;
    x5 = mirrored ? px0 : px5;
    
    p0 = [x0, 0 + space / 2];
    p1 = [x1, 0 + space / 2];
    p2 = [x2, length / 2];
    p3 = [x0, length / 2];
    
    pointsSingle = [p0, p1, p2, p3];
    polygon(pointsSingle);
    
    p5 = [x3, 0];
    p6 = [x4, length / 2 - space / 2];
    p7 = [x5, length / 2 - space / 2];
    p8 = [x5, 0];
    
    pointsLowerHalf = [p5, p6, p7, p8];
    polygon(pointsLowerHalf);
}


function distance2(angle) = (sin(angle) * wallLength / 2) / sin(45 - abs(angle));

module hexes2(angle, mirrored) {
    if (angle < 22) {
        d0 = distance2(angle);
        d1 = distance2(angle + shortSide/ 2);
        d2 = distance2(angle + arms + shortSide / 2);
        
        d3 = distance2(angle + shortSide/ 2 + spacing);
        d4 = distance2(angle + shortSide/ 2 + spacing + arms);
        d5 = distance2(angle + shortSide + spacing + arms);
        
        if (d3 > -wallLength / 2){
            hexHalf(length, d0, d1, d2, d3, d4, d5, spacingAbs, mirrored);
            
            mirror([0, 1, 0]) {
                translate([0, -length, 0]) {
                    hexHalf(length, d0, d1, d2, d3, d4, d5, spacingAbs, mirrored);
                }
            }
        }
        
        hexes2(angle + arms + shortSide + spacing, !mirrored);
    }
}

module sideSlices() {
    for (n = [0 : 1 : wallLength / length - spacing]) {
        translate([wallLength / 2, length * n, 0]) {
            hexes2(-45, false);
        }
    }
}

difference() {
//difference() {
    //sideWall(wallLength, wallLength, wallThickness);


    color("white") {
        linear_extrude(height=4) {
            sideSlices();
        }
    }
//}
    cube([60, 160, 10]);
    cube([160, 120, 10]);

    translate([100, 0, 0]) {
        cube([60, 160, 10]);
    }
}

