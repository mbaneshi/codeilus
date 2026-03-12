import os
from pathlib import Path

class FileReader:
    def __init__(self, path):
        self.path = path

    def read(self):
        return Path(self.path).read_text()

def process(reader):
    content = reader.read()
    return content.upper()
