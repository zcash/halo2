# Lookup tables

In normal programs, you can trade memory for CPU to improve performance, by pre-computing
and storing lookup tables for some part of the computation. We can do the same thing in
halo2 circuits!

A lookup table can be thought of as enforcing a *relation* between variables, where the relation is expressed as a table.
Assuming we have only one lookup argument in our constraint system, the total size of tables is constrained by the size of the circuit:
each table entry costs one row, and it also costs one row to do each lookup.

TODO
