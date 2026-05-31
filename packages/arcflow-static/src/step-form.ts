export interface ConversationTurn {
  role: "user" | "assistant";
  content: string;
}

export class StepForm {
  private turns: ConversationTurn[] = [];

  addTurn(role: ConversationTurn["role"], content: string): this {
    this.turns.push({ role, content });
    return this;
  }

  toInitialState(): Record<string, unknown> {
    return { conversation_turns: this.turns };
  }
}
