$fa = 0.1;
$fs = 0.4;

include <utils.scad>;
include <parameters.scad>;

// Render half of a hexagon shape
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

// Render a line of side pattern hexagons.
module sideHexes(angle, mirrored) {
    if (angle < 22) {
        d0 = distance(angle, wallLength / 2);
        d1 = distance(angle + shortSide/ 2, wallLength / 2);
        d2 = distance(angle + armsDistance + shortSide / 2, wallLength / 2);
        
        d3 = distance(angle + shortSide/ 2 + spacing, wallLength / 2);
        d4 = distance(angle + shortSide/ 2 + spacing + armsDistance, wallLength / 2);
        d5 = distance(angle + shortSide + spacing + armsDistance, wallLength / 2);
        
        if (angle > -23){
            hexHalf(hexLength, d0, d1, d2, d3, d4, d5, spacingAbs, mirrored);
            
            mirror([0, 1, 0]) {
                translate([0, -hexLength, 0]) {
                    hexHalf(hexLength, d0, d1, d2, d3, d4, d5, spacingAbs, mirrored);
                }
            }
        }
        
        sideHexes(angle + armsDistance + shortSide + spacing, !mirrored);
    }
}

// Render the hexagons for the enclosure side walls.
module sidePattern() {
    linear_extrude(height=hexThickness) {
        for (n = [0 : 1 : wallLength / hexLength - spacing]) {
            translate([wallLength / 2, hexLength * n, 0]) {
                sideHexes(-45, true);
            }
        }
    }
}

// Hexes at the edges won't get any light passing through them by default.
// This merges them with some of the neighbouring hexes.
module hexEdgeFix() {
    translate([0, 0, 0]) {
        cube([sideEdgeFixWidth, wallLength, 2 * (betweenThickness + solidThickness)]);
    }
    
    translate([wallLength - sideEdgeFixWidth, 0, 0]) {
        cube([sideEdgeFixWidth, wallLength, 2 * (betweenThickness + solidThickness)]);
    }
}

// Render the hex pattern for the side wall.
module sideHexPattern(hideHexes, hideSolid) {
    if (!hideSolid) {
        // do not show hex shapes
        color("black") {
            difference() {
                solidWalls();
                sidePattern();
                hexEdgeFix();
            }
        }
    } 
    
    if(!hideHexes) {
        // only show hex shapes
        color("white") {
            sidePattern();
            translate([0, 0, solidThickness]) {
                cube([wallLength, wallLength, betweenThickness]);
            }
            
            translate([0, 0, solidThickness * 2 + betweenThickness]) {
                cube([wallLength, wallLength, betweenThickness]);
            }
            
            hexEdgeFix();
        }
    }
}

// Renders the front enclosure wall.
module sideWall(hideHexes, hideSolid) {   
    difference() {
        sideHexPattern(hideHexes, hideSolid);
        
        // apply side bevels
        allSidesBevelTool(wallLength, hexThickness);
    }  
}

sideWall(false, true);
