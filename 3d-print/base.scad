inner = 20;
block = 0.5;
thick = 1.5;


length = 10;
support = 4;

outer = inner + thick;
middle = inner + block;
full_length = length + support;

$fn = 64;

difference() {
    cylinder(h = full_length, d = outer);
    translate([0,0,full_length - support]) cylinder(h = full_length, d = middle);
    union() {
        cylinder(h = full_length * 4, d = inner, center = true);
    }
}
