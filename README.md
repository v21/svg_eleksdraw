# svg_eleksdraw

## Usage

This tool will convert SVG files to gcode for use with an Eleksdraw pen plotter.

To use, call with the path to a svg file:

`svg_eleksdraw drawing.svg`

By default, it will then create a `drawing.gcode` (which can be fed to your plotter, ie by using Universal Gcode Sender), and an accompanying `drawing.stripped.svg`, which will provide a preview of the plot.

## Things to know:

- it's not well tested
- it converts beziers to a series of straight lines at high precision
- it doesn't support arcs (yet?)
- it doesn't prevent you from exceeding the mechanical limits of the Eleksdraw
- but it will prevent any paths from exceeding the viewBox - you can set your limits there and be safe
- it will start the plot by lifting the pen to it's fullest extent, and moving around the limits of the viewBox. This helps to check that no mechanical limits will be exceeded during a plot, and to let you preview the plotting area. It then pauses for 3 seconds, to give you a chance to cancel if something will go wrong, and starts.

## Command line options:

- `-o` or `--output-dir` : set output directory (by default, this will be the same as the directory the input files are in)
- `-u` or `--pen-up` : pen height when raised. Set this to prevent excessive travel (by default, 0)
- `-d` or `--pen-down` : pen height when lowered. Set this to prevent excessive travel (by default, 100)
- `-s` or `--max-speed` : maximum movement speed when drawing lines (by default, 10000)
