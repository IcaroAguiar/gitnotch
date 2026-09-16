export class WorkspaceEpochAcceptor {
  #current: number | null = null;

  accept(next: number): boolean {
    if (this.#current !== null && next < this.#current) return false;
    this.#current = next;
    return true;
  }

  current(): number | null {
    return this.#current;
  }
}
