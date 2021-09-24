// Utility modules and functions

// Compute the coordinates of a point using on an angled line with a specified distance d from the origin.
function point(angle, d) = [(sin(angle) * d) / sin(45 - abs(angle)), d];

// Compute the distance of a point on an angled line with a specified distance d from the origin on that line.
function distance(angle, d) = (sin(angle) * d) / sin(45 - abs(angle));

// Tool to bevel a single side of an enclosure wall.
module sideBevelTool(wallLength, height) {
    translate([0, 0, hexThickness]) {
        rotate([0, 90, 0]) {
            linear_extrude(height=wallLength) {
                polygon([[-1,-1], [hexThickness + 1, -1], [hexThickness + 1, hexThickness + 1]]);
            }
        }
    }
}

// Tool to bevel all four sides of an enclosure wall.
module allSidesBevelTool(wallLength, hexThickness) {
    sideBevelTool(wallLength, hexThickness);
    
    
    translate([0, wallLength, 0]) {
        mirror([0, 1, 0]) {
            sideBevelTool(wallLength, hexThickness);
        }
    }
    
    translate([0, wallLength, 0]) {
        rotate([0, 0, -90]) {
            sideBevelTool(wallLength, hexThickness);
        }
    }
    
    mirror([1, 0, 0]) {
        translate([-wallLength, wallLength, 0]) {
            rotate([0, 0, -90]) {
                sideBevelTool(wallLength, hexThickness);
            }
        }
    }
}

// Render the wall parts made of solid material.
module solidWalls() {
    color("black") {
        cube([wallLength, wallLength, solidThickness]); 
        
        translate([0, 0, solidThickness + betweenThickness]) {
            cube([wallLength, wallLength, solidThickness]);
        }
        
        translate([0, 0, 2 * solidThickness + 2 * betweenThickness]) {
            cube([wallLength, wallLength, solidThickness]);
        }
    }
}