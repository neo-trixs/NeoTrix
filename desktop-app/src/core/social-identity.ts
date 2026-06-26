/**
 * Social Identity — 社交身份模块
 *
 * 架构: Nostr 密钥对身份模型
 * - 首次启动自动生成 Ed25519 密钥对
 * - 用户 ID = 公钥 hex 前 16 位 (人类友好)
 * - 联系人列表 + 简档本地存储
 * - 最终对接 Nostr relay / Hyperswarm P2P
 */



/** 模拟 Ed25519 密钥对 (生产环境替换为 @noble/ed25519 或 Tauri 原生) */
export interface KeyPair {
  pubkey: string;   // hex
  seckey: string;   // hex (加密存储)
}

/** 用户简档 (Nostr kind 0) */
export interface Profile {
  name: string;
  about: string;
  picture: string;   // data URI or URL
  displayName?: string;
  nip05?: string;
}

/** 联系人 (Nostr kind 3) */
export interface Contact {
  pubkey: string;
  alias: string;
  addedAt: number;
  relayHint: string;
  trustScore: number;     // 0-1
  online: boolean;
  lastSeen: number;
  unread: number;
}

/**
 * 从 Ed25519 pubkey 确定性派生 VSA 4096-bit 向量
 *
 * 统一身份: NeoTrix 的认知层身份 (VSA) = 社交层身份 (Ed25519) 的确定性映射
 * 使得 AgentCommunicationBus 可通过 VSA 向量原生路由到社交联系人
 */
export function vsaFromPubkey(pubkey: string): string {
  // FNV-1a 风格种子
  let h = 0x811c9dc5;
  for (let i = 0; i < pubkey.length; i++) {
    h = Math.imul(h ^ pubkey.charCodeAt(i), 0x01000193) >>> 0;
  }
  // 扩展为 1024 hex chars = 4096 bits
  let out = '';
  let s = h;
  for (let i = 0; i < 64; i++) {
    s = (s * 1103515245 + 12345) >>> 0;
    out += s.toString(16).padStart(8, '0');
  }
  return out;
}

/** 社交身份单例 */
export class SocialIdentity {
  private _keypair: KeyPair | null = null;
  private _profile: Profile | null = null;
  private _contacts: Contact[] = [];
  private _initialized = false;

  get initialized(): boolean { return this._initialized; }
  get pubkey(): string { return this._keypair?.pubkey ?? ''; }
  get shortId(): string { return this.pubkey ? this.pubkey.slice(0, 16) : ''; }
  get keypair(): KeyPair | null { return this._keypair; }
  get profile(): Profile | null { return this._profile; }
  get contacts(): Contact[] { return [...this._contacts]; }

  /** 首次启动初始化: 生成密钥对 + 默认简档 */
  init(): void {
    const saved = this._load();
    if (saved) {
      this._keypair = saved.keypair;
      this._profile = saved.profile;
      this._contacts = saved.contacts;
      this._initialized = true;
      return;
    }

    // 首次启动: 生成密钥对
    this._keypair = this._generateKeypair();
    this._profile = {
      name: `User_${this.shortId.slice(0, 8)}`,
      about: 'NeoTrix social user',
      picture: '',
    };
    this._contacts = [];
    this._save();
    this._initialized = true;

    // 添加默认联系人 (演示用)
    this._seedDemoContacts();
  }

  private _seedDemoContacts(): void {
    this.addContact({
      pubkey: 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1',
      alias: 'Alice',
      relayHint: 'wss://relay.damus.io',
    });
    this.addContact({
      pubkey: 'b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2',
      alias: 'Bob',
      relayHint: 'wss://relay.nostr.band',
    });
    this.addContact({
      pubkey: 'c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3',
      alias: 'Carol',
      relayHint: '',
    });
  }

  updateProfile(p: Partial<Profile>): void {
    if (!this._profile) return;
    Object.assign(this._profile, p);
    this._save();
  }

  getContact(pubkey: string): Contact | undefined {
    return this._contacts.find(c => c.pubkey === pubkey);
  }

  addContact(c: { pubkey: string; alias: string; relayHint?: string }): Contact {
    const existing = this._contacts.find(x => x.pubkey === c.pubkey);
    if (existing) return existing;
    const contact: Contact = {
      pubkey: c.pubkey,
      alias: c.alias,
      addedAt: Date.now(),
      relayHint: c.relayHint ?? '',
      trustScore: 0.5,
      online: false,
      lastSeen: Date.now(),
      unread: 0,
    };
    this._contacts.push(contact);
    this._save();
    return contact;
  }

  removeContact(pubkey: string): void {
    this._contacts = this._contacts.filter(c => c.pubkey !== pubkey);
    this._save();
  }

  setContactOnline(pubkey: string, online: boolean): void {
    const c = this._contacts.find(x => x.pubkey === pubkey);
    if (c) { c.online = online; c.lastSeen = Date.now(); }
  }

  incrementUnread(pubkey: string): void {
    const c = this._contacts.find(x => x.pubkey === pubkey);
    if (c) c.unread++;
  }

  clearUnread(pubkey: string): void {
    const c = this._contacts.find(x => x.pubkey === pubkey);
    if (c) c.unread = 0;
  }

  /** 从 pubkey 派生 VSA 4096-bit 向量 (hex) */
  vsaOf(pubkey: string): string {
    return vsaFromPubkey(pubkey);
  }

  /** 当前身份的 VSA 向量 */
  get vsaId(): string {
    return this.vsaOf(this.pubkey);
  }

  /** 模拟签名 (生产环境用 @noble/ed25519) */
  sign(_content: string): string {
    return `${this.pubkey.slice(0, 16)}_sig_${Date.now().toString(36)}`;
  }

  private _generateKeypair(): KeyPair {
    // 模拟 Ed25519 密钥对
    const chars = '0123456789abcdef';
    const randHex = (n: number) => Array.from({ length: n }, () => chars[Math.floor(Math.random() * 16)]).join('');
    return {
      pubkey: randHex(64),
      seckey: randHex(128),
    };
  }

  private _storageKey = 'neotrix_social_identity';

  private _save(): void {
    try {
      localStorage.setItem(this._storageKey, JSON.stringify({
        keypair: this._keypair,
        profile: this._profile,
        contacts: this._contacts,
      }));
    } catch (e) { if (e instanceof Error) console.warn('[SocialIdentity]', e.message); }
  }

  private _load(): { keypair: KeyPair; profile: Profile; contacts: Contact[] } | null {
    try {
      const raw = localStorage.getItem(this._storageKey);
      return raw ? JSON.parse(raw) : null;
    } catch (e) { if (e instanceof Error) console.warn('[SocialIdentity]', e.message); return null; }
  }
}

export const socialIdentity = new SocialIdentity();
