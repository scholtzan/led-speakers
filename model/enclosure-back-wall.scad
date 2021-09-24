include <utils.scad>;
include <parameters.scad>;
include <enclosure-front-wall.scad>;

// Render the back wall which is based on the front wall pattern.
module backWall(hideHexes, hideSolid) {
    difference() {
        difference() {
            frontHexPattern(hideHexes, hideSolid);
            
            // apply side bevels
            allSidesBevelTool(wallLength, hexThickness);
        }
        
        // holes for banana adapters
        translate([adapterEdgeDistance, wallLength / 2 + adapterDistance + ledAdapterWidth / 2, 0]) {
            linear_extrude(height = hexThickness) {
                circle(d=bananaAdapterDiameter);
            }
        }
        
        translate([adapterEdgeDistance, wallLength / 2 - adapterDistance - ledAdapterWidth / 2, 0]) {
            linear_extrude(height = hexThickness) {
                circle(d=bananaAdapterDiameter);
            }
        }
        
        // hole for LED adapter
        translate([adapterEdgeDistance, wallLength / 2 - ledAdapterWidth / 2, 0]) {
            cube([ledAdapterHeight, ledAdapterWidth, hexThickness]);
        }
    }
    
    // center square
    if (!hideHexes) {
        translate([wallLength / 2 - hexLength / 2, wallLength / 2  - hexLength / 2, 0]) {
            cube([hexLength, hexLength, hexThickness]);
        }
    }
}

backWall(false, true);
