# 社媒通信能力拆解分析

## 发现的社媒通信相关类型

### 1. Channel 枚举 (unified_types.rs:99)

```rust
pub enum Channel {
    // ── 贸易平台 (外贸特有) ──
    Alibaba, MadeInChina, GlobalSources,
    
    // ── 广告渠道 (外贸特有) ──
    GoogleAds, FacebookAds, LinkedInAds,
    
    // ── 通信渠道 (通用) ──
    WhatsApp, Email, WeChat, LinkedIn,
    
    // ── 网站渠道 (通用) ──
    WebsiteForm, WebsiteChat, SEOOrganic,
    
    // ── 其他 (混合) ──
    Exhibition, Referral, ColdCall,
    CustomsData, Other(String),
}
```

### 2. InteractionType 枚举 (nt_trade_crm.rs:322)

```rust
pub enum InteractionType {
    // ── 通信类型 (通用) ──
    Email,
    Phone,
    WhatsApp,
    Meeting,
    VideoCall,
    
    // ── 业务类型 (外贸特有) ──
    Inquiry,
    Quotation,
    Contract,
    System,
}
```

### 3. CommunicationChannel (sales_coaching.rs)

```rust
pub enum CommunicationChannel {
    Email,
    Social,      // 社交媒体
    Phone,
    Meeting,
}
```

### 4. ContactInfo (unified_types.rs:480)

```rust
pub struct ContactInfo {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub whatsapp: Option<String>,
    pub wechat: Option<String>,
    pub linkedin: Option<String>,
}
```

## 能力分类

### ✅ 通用能力 (应提取到 nt_act)

| 类型 | 位置 | 说明 |
|------|------|------|
| `CommunicationChannel` | nt_act/message/ | 通信渠道枚举 |
| `InteractionType` (通信部分) | nt_act/message/ | Email/Phone/WhatsApp/Meeting/VideoCall |
| `ContactInfo` | nt_act/contact/ | 联系人信息 |
| `PlatformFeature::WhatsAppSync` | nt_act/pipeline/ | 平台同步能力 |

### 🔶 外贸特有能力 (保留)

| 类型 | 位置 | 说明 |
|------|------|------|
| `Channel::Alibaba` 等 | nt_act_trade | 贸易平台渠道 |
| `Channel::GoogleAds` 等 | nt_act_trade | 广告渠道 |
| `InteractionType::Inquiry` 等 | nt_act_trade | 业务交互类型 |

## 重构建议

### Phase 1: 提取通用通信能力

```
nt_act/
├── communication/
│   ├── mod.rs
│   ├── channel.rs          # CommunicationChannel enum
│   ├── interaction.rs      # InteractionType (通信部分)
│   ├── contact.rs          # ContactInfo struct
│   └── social_media.rs     # SocialMediaPlatform enum
```

### Phase 2: 定义社媒平台枚举

```rust
/// 社交媒体平台
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocialMediaPlatform {
    WhatsApp,
    WeChat,
    LinkedIn,
    Facebook,
    Twitter,
    Instagram,
    Telegram,
    Line,
    Viber,
}

/// 通信渠道
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommunicationChannel {
    Email,
    Phone,
    Social(SocialMediaPlatform),
    Meeting,
    VideoCall,
    Website,
}

/// 联系人信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContactInfo {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub social: HashMap<SocialMediaPlatform, String>,
}
```

### Phase 3: nt_act_trade 引用通用能力

```rust
// nt_act_trade/unified_types.rs 改为:
pub use nt_act::communication::{
    CommunicationChannel, ContactInfo, SocialMediaPlatform,
};

// 保留外贸特有渠道
pub enum Channel {
    // 通用通信渠道 (引用)
    Email, WhatsApp, WeChat, LinkedIn,
    
    // 外贸特有渠道 (保留)
    Alibaba, MadeInChina, GlobalSources,
    GoogleAds, FacebookAds, LinkedInAds,
    // ...
}
```

## 收益

| 收益 | 说明 |
|------|------|
| 复用性 | 其他Agent（如社交Agent、客服Agent）可复用通信能力 |
| 一致性 | 所有Agent使用相同的通信渠道枚举 |
| 扩展性 | 新增社媒平台只需修改一处 |
