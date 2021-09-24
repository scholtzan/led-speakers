$fa = 0.1;
$fs = 0.4;

include <utils.scad>;
include <parameters.scad>;

// Render hexagon shapes;
module frontHexes(angle, h, l, xMirrored) {
    if (angle < 22) {
        d0 = point(angle, xMirrored ? l : l + h);
        d1 = point(
            angle + shortSide / 2 + armsDistance, 
            xMirrored ? l : l + h
        );
        d2 = point(
            angle + shortSide / 2, 
            xMirrored ? l + h - spacingAbs / 2 : l + spacingAbs / 2);
        d3 = point(
            angle, 
            xMirrored ? l + h - spacingAbs / 2 : l + spacingAbs / 2
        );
        
        if (angle > -23) {
            polygon([d0, d1, d2, d3]);
            
            d4 = point(
                angle + spacing + shortSide / 2, 
                xMirrored ? l + h : l
            );
            d5 = point(
                angle + spacing + shortSide + armsDistance, 
                xMirrored ? l + h : l
            );
            d6 = point(
                angle + spacing + shortSide + armsDistance, 
                xMirrored ? l + spacingAbs / 2 : l + h - spacingAbs / 2
            );
            d7 = point(
                angle + spacing + shortSide / 2 + armsDistance, 
                xMirrored ? l + spacingAbs / 2 : l + h - spacingAbs / 2
            );
            
            polygon([d4, d5, d6, d7]);
        }
        
        frontHexes(angle + armsDistance + shortSide + spacing, h, l, !xMirrored);
    }
}

// Render a slice of the hex patterns
module frontSlices(ofst, xMirrored) {
    if (ofst < wallLength / 2) {
        frontHexes(-45, hexLength / 2, ofst, xMirrored);
        frontSlices(ofst + hexLength / 2, !xMirrored);
    }
}

// Render the front hex pattern
module hexPattern() {
    translate([wallLength / 2, wallLength / 2, 0]) {
        color("white") {
            linear_extrude(height = hexThickness) {
                // 0.01 required otherwise nothing will be rendered
                frontSlices(0.01, true);
                
                mirror([0, 1, 0]) {
                    frontSlices(0.01, true);
                }
                mirror([1, 1, 0]) {
                    frontSlices(0.01, true);
                }
                mirror([-1, 1, 0]) {
                    frontSlices(0.01, true);
                }
            }
        }
    }
}

// Render the hex pattern for the front wall.
module frontHexPattern(hideHexes, hideSolid) {
    if (!hideSolid) {
        // do not show hex shapes
        difference() {
            solidWalls();
            hexPattern();
        }
    } 
    
    if(!hideHexes) {
        // only show hex shapes
        color("white") {
            hexPattern();
            translate([0, 0, solidThickness]) {
                cube([wallLength, wallLength, betweenThickness]);
            }
            
            translate([0, 0, solidThickness * 2 + betweenThickness]) {
                cube([wallLength, wallLength, betweenThickness]);
            }
        }
    }
}

// Renders the front enclosure wall.
module frontWall(hideHexes, hideSolid) {   
    difference() {
        difference() {
            difference() {
                difference() {
                    frontHexPattern(hideHexes, hideSolid);
                    
                    // apply side bevels
                    allSidesBevelTool(wallLength, hexThickness);
                }
                
                // rect indent
                translate([
                    wallLength / 2 - frontRectLength / 2, 
                    wallLength / 2 - frontRectLength / 2, 
                    3 * solidThickness + 2 * betweenThickness]) {
                    cube([frontRectLength, frontRectLength, hexThickness]);
                }
                
                // circle indent
                translate([
                    wallLength / 2, 
                    wallLength / 2, 
                    3 * solidThickness + 2 * betweenThickness]) {
                    linear_extrude(height=hexThickness) {
                        circle(d=frontCircleDiameter);
                    }
                }
            }
            
            // speaker cone hole
            translate([wallLength / 2, wallLength / 2 , 0]) {
                linear_extrude(height = hexThickness) {
                    circle(d=speakerConeDiameter);
                }
            }
        }
        
        // screw mounting holes
        for (i = [-1 : 2 : 1]) {
            for (j = [-1 : 2 : 1]) {
                translate([
                    wallLength / 2 - screwCenterDistance * i, 
                    wallLength / 2 - screwCenterDistance * j,
                    0
                ]) {
                    linear_extrude(height=hexThickness) {
                        circle(d=screwDiameter);
                    }
                }
            }
        }
    }
}

// frontWall(true, false);

