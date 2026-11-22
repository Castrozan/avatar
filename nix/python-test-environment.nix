{ pkgs }:
pkgs.python312.withPackages (pythonPackages: [
  pythonPackages.pytest
  pythonPackages.hypothesis
  pythonPackages.numpy
  pythonPackages.websockets
])
