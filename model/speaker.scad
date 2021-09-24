include <enclosure-front-wall.scad>;
include <enclosure-side-wall.scad>;
include <enclosure-back-wall.scad>;
include <cover.scad>;
include <stand.scad>;

// If false, renders hex shapes;
// toggle when exporting different models to enable slicing with dual nozzles
hideHexes = false;

// If false, only renders solid material between hex shapes;
// toggle when exporting different models to enable slicing with dual nozzles
hideSolid = false;

// Render all speaker parts.
module renderAssembledSpeaker() {
    renderBackWall();
    
    translate([0, 0, -wallLength + 2 * hexThickness]) {
        mirror([0, 0, 1]) {
            renderFrontWall();
        }
    }
    
    translate([hexThickness, 0, hexThickness]) {
        rotate([-90, 0, 90]) {
            renderSideWall();
        }
    }
    
    translate([wallLength - hexThickness, 0, hexThickness]) {
        mirror([1, 0, 0]) {
            rotate([-90, 0, 90]) {
                renderSideWall();
            }
        }
    }
    
    rotate([0, 0, 90]) {
        mirror([0, 1, 0]) {
            translate([hexThickness, 0, hexThickness]) {
                rotate([-90, 0, 90]) {
                    renderSideWall();
                }
            }
        }
    }
    
    translate([0, wallLength, 0]) {
        mirror([0, 1, 0]) {
            rotate([0, 0, 90]) {
                mirror([0, 1, 0]) {
                    translate([hexThickness, 0, hexThickness]) {
                        rotate([-90, 0, 90]) {
                            renderSideWall();
                        }
                    }
                }
            }
        }
    }
    
    
    color("black") {
        translate([wallLength/2, wallLength/2, frontRectLength * 5 - wallLength]) {
            mirror([0, 0, 1]) {
                speakerCover();
            }
        }
        
        translate([wallLength - standWidth, wallLength + standHeight * 1.75, -standLength + standRimWidth * 2]) {
            rotate([0, -90, 180]) {
                speakerStand();
            }
        }
    }
}

// Render the front wall.
module renderFrontWall() {
    frontWall(hideHexes, hideSolid);
}

// Render the back wall.
module renderBackWall() {
    backWall(hideHexes, hideSolid);
}

// Render a single side wall.
module renderSideWall() {
    sideWall(hideHexes, hideSolid);
}


////// Render

renderAssembledSpeaker();
//renderFrontWall();
//renderSideWall();
//renderBackWall();