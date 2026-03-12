import { readFile } from 'fs/promises';

interface Reader {
  read(): Promise<string>;
}

class FileReader implements Reader {
  constructor(private path: string) {}

  async read(): Promise<string> {
    return readFile(this.path, 'utf-8');
  }
}

export function process(reader: Reader): Promise<string> {
  return reader.read();
}
