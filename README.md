# Oter - GIS Data Conversion Tool

[![Build Status](https://img.shields.io/github/actions/workflow/status/elamandeep/oter/ci.yml?branch=main)](https://github.com/elamandeep/oter/actions)
[![PyPI version](https://img.shields.io/pypi/v/oter)](https://pypi.org/project/oter/)
[![License](https://img.shields.io/github/license/elamandeep/oter)](https://github.com/elamandeep/oter/blob/main/LICENSE)

Oter is a command-line tool designed to convert spatial data into various formats such as .shp, .topojson, .geojson, and more. As it is still in development, performance improvements and additional features like merging GeoJSON files and compression are planned for future releases.

## Features

- Convert spatial data between different formats:
  - Shapefile (.shp)
  - TopoJSON (.topojson)
  - GeoJSON (.geojson)
  - And more
- Planned features:
  - Merging multiple GeoJSON files into one
  - Compressing output files

## Installation

You can install Oter via pip for Python  for Rust.

### Python

> pip install oter



### Usage

- After installing Oter, you can use it from the command line.
- To see the available commands, run:


> oter -h

# Example Commands

- Convert Shapefile to GeoJSON

> oter --shpgeojson ./folder_name


# Development
- Oter is in the early stages of development. 
- We welcome contributions and feedback from the community to help improve its performance and add new features.
-  Feel free to open issues or submit pull requests on our GitHub repository.

## Contributing

1. Fork the repository
2. Create your feature branch (git checkout -b feature/my-new-feature)
3. Commit your changes (git commit -am 'Add some feature')
4. Push to the branch (git push origin feature/my-new-feature)
5. Create a new Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

For questions or feedback, please contact [elamandeep](mailto:deep.aman6174@gmail.com).