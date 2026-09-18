# NeoTrixBrain 医疗知识图谱吸收报告

**吸收时间**: 2026-09-18
**源文件**: `/Volumes/NeoTrixBrain/working/causal_graph.json`
**KB Session**: `absorption_neotrixbrain_medical`

## 图谱统计

| 指标 | 值 |
|------|-----|
| 节点数 | 60 |
| 边数 | 50 |
| 置信度 (Remedy→Condition) | 0.7 |
| 置信度 (Condition→Relief) | 0.56 |
| 主题分类 | 7 |

## 主题分类

### 1. Cold, cough & allergy (感冒/咳嗽/过敏)
- Cough, Sore throat, Common cold, Nasal congestion
- 疗法: Honey, Fluids and rest, Humidifier/steam, Saline nasal rinse, Salt-water gargle

### 2. Pain, fever & inflammation (疼痛/发热/炎症)
- Fever, Muscle & joint pain, Menstrual cramps, Headache
- 疗法: Fluids and rest, Cool compress, Ice and elevation, Heating pad, Rest in dark/quiet room

### 3. Stomach & digestion (消化系统)
- Diarrhea, Nausea & vomiting, Constipation, Heartburn, Indigestion
- 疗法: Fluids and rest, Oral rehydration salts, Dietary fiber, Elevate head of bed, Small frequent bland meals

### 4. Skin & wounds (皮肤/伤口)
- Insect bites, Rash & itching, Poison ivy, Dry skin, Burns, Wounds & cuts, Athlete's foot, Acne
- 疗法: Cool compress, Ice and elevation, Oatmeal/cool bath, Cool running water, Clean and cover, Aloe Vera, Keep feet clean

### 5. Eyes, ears & mouth (眼/耳/口腔)
- Eye allergies, Eye irritation, Earache, Canker & cold sores
- 疗法: Cool compress, Heating pad, Humidifier/steam, Salt-water gargle

### 6. Sleep, stress & general (睡眠/压力/一般)
- Dehydration, Hemorrhoids, Sleeplessness
- 疗法: Oral rehydration salts, Dietary fiber, Sitz bath, Sleep hygiene, Dark/quiet room

### 7. Infections (OTC-treatable) (可OTC治疗的感染)
- Pinworm, Athlete's foot & ringworm
- 疗法: Hygiene measures, Keep feet clean

## 因果链结构

```
Remedy (0.7) → Condition (0.56) → Symptom Relief Expected
```

所有疗法均指向具体病症，所有病症均以 "symptom relief expected" 为终点目标。

## 关键知识提取

### 疗法→病症映射

| 疗法 | 适用病症数 | 代表病症 |
|------|-----------|----------|
| Fluids and rest | 4 | Common cold, Fever, Diarrhea, Nausea |
| Cool compress | 3 | Fever, Insect bites, Eye allergies |
| Humidifier/steam | 3 | Nasal congestion, Cough, Common cold |
| Heating pad | 3 | Menstrual cramps, Muscle pain, Earache |
| Small frequent bland meals | 3 | Nausea, Indigestion, Diarrhea |
| Aloe Vera | 3 | Burns, Acne, Dry skin |
| Salt-water gargle | 3 | Sore throat, Common cold, Canker sores |
| Oatmeal/cool bath | 3 | Rash, Poison ivy, Dry skin |
| Honey | 3 | Cough, Sore throat, Common cold |

### 自然疗法 (Natural Remedies)
- **Aloe Vera**: 烧伤、痤疮、干燥皮肤 (抗炎+保湿)
- **Boswellia**: 肌肉关节痛 (抗炎)
- **Bromelain**: 肌肉关节痛 + 鼻塞 (消肿)

## KB 落盘

- **命名空间**: `experience`
- **类别**: `medical_knowledge`
- **优先级**: `medium`
- **证据链**: `/Volumes/NeoTrixBrain/working/causal_graph.json`
