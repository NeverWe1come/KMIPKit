import type * as vscode from "vscode";

const SECRET_PREFIX = "kmipkit.profile.";

export class KmipSecretStore {
  constructor(private readonly secrets: vscode.SecretStorage) {}

  async setProfileSecret(profileName: string, key: string, value: string): Promise<void> {
    await this.secrets.store(`${SECRET_PREFIX}${profileName}.${key}`, value);
  }

  async getProfileSecret(profileName: string, key: string): Promise<string | undefined> {
    return this.secrets.get(`${SECRET_PREFIX}${profileName}.${key}`);
  }

  async deleteProfileSecret(profileName: string, key: string): Promise<void> {
    await this.secrets.delete(`${SECRET_PREFIX}${profileName}.${key}`);
  }
}
