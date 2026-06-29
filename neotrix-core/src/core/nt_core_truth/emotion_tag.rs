use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct EmotionTag {
    pub valence: f64,
    pub arousal: f64,
    pub dominance: f64,
    pub primary_emotion: String,
    pub trigger_source: String,
    pub word_hits: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EmotionLexicon {
    valence_map: HashMap<&'static str, (f64, f64, f64, &'static str)>,
    negation_markers: Vec<&'static str>,
    intensity_modifiers: HashMap<&'static str, f64>,
}

impl Default for EmotionLexicon {
    fn default() -> Self {
        let mut valence_map: HashMap<&'static str, (f64, f64, f64, &'static str)> = HashMap::new();
        for (word, v, a, d, e) in Self::lexicon_data() {
            valence_map.insert(word, (v, a, d, e));
        }
        Self {
            valence_map,
            negation_markers: Self::negation_data(),
            intensity_modifiers: Self::intensity_data(),
        }
    }
}

impl EmotionLexicon {
    fn negation_data() -> Vec<&'static str> {
        vec![
            "not", "no", "never", "neither", "nor", "cannot", "cant", "dont", "doesnt", "didnt",
            "wont", "wouldnt", "shouldnt", "couldnt", "isnt", "arent", "wasnt", "werent", "havent",
            "hasnt", "hadnt", "nobody", "nothing", "nowhere", "none",
        ]
    }

    fn intensity_data() -> HashMap<&'static str, f64> {
        let mut m = HashMap::new();
        m.insert("very", 1.5);
        m.insert("extremely", 2.0);
        m.insert("incredibly", 2.0);
        m.insert("unbelievably", 2.0);
        m.insert("somewhat", 0.5);
        m.insert("slightly", 0.3);
        m.insert("barely", 0.2);
        m.insert("hardly", 0.2);
        m.insert("deeply", 1.8);
        m.insert("profoundly", 1.9);
        m.insert("absolutely", 1.8);
        m.insert("totally", 1.6);
        m.insert("completely", 1.6);
        m.insert("utterly", 1.7);
        m.insert("quite", 1.3);
        m.insert("rather", 1.2);
        m.insert("pretty", 1.2);
        m.insert("fairly", 1.1);
        m.insert("moderately", 0.6);
        m.insert("truly", 1.5);
        m.insert("genuinely", 1.4);
        m.insert("increasingly", 1.3);
        m.insert("incredibly", 2.0);
        m.insert("remarkably", 1.7);
        m.insert("exceedingly", 1.9);
        m.insert("intensely", 1.9);
        m.insert("overwhelmingly", 2.0);
        m.insert("highly", 1.5);
        m.insert("acutely", 1.6);
        m.insert("awfully", 1.4);
        m.insert("terribly", 1.5);
        m.insert("dreadfully", 1.6);
        m
    }

    fn lexicon_data() -> Vec<(&'static str, f64, f64, f64, &'static str)> {
        vec![
            // === BASIC EMOTIONS — Anger ===
            ("anger", -0.85, 0.90, 0.60, "anger"),
            ("fury", -0.90, 0.95, 0.65, "anger"),
            ("rage", -0.92, 0.97, 0.70, "anger"),
            ("irritated", -0.55, 0.50, 0.30, "anger"),
            ("annoyed", -0.40, 0.40, 0.25, "anger"),
            ("frustrated", -0.60, 0.65, 0.20, "anger"),
            ("fuming", -0.85, 0.90, 0.60, "anger"),
            ("seething", -0.88, 0.93, 0.62, "anger"),
            ("livid", -0.90, 0.94, 0.63, "anger"),
            ("wrath", -0.88, 0.92, 0.68, "anger"),
            ("hostile", -0.75, 0.80, 0.55, "anger"),
            ("bitter", -0.60, 0.30, -0.10, "anger"),
            ("resentful", -0.65, 0.40, 0.00, "anger"),
            ("mad", -0.70, 0.75, 0.50, "anger"),
            ("irate", -0.87, 0.93, 0.64, "anger"),
            ("infuriated", -0.91, 0.96, 0.66, "anger"),
            ("enraged", -0.92, 0.97, 0.68, "anger"),
            ("agitated", -0.50, 0.70, 0.10, "anger"),
            ("exasperated", -0.65, 0.72, 0.15, "anger"),
            ("cross", -0.45, 0.55, 0.20, "anger"),
            // === BASIC EMOTIONS — Fear ===
            ("fear", -0.80, 0.85, -0.40, "fear"),
            ("terrified", -0.90, 0.95, -0.50, "fear"),
            ("anxious", -0.60, 0.70, -0.30, "fear"),
            ("worry", -0.50, 0.55, -0.25, "fear"),
            ("scared", -0.75, 0.80, -0.35, "fear"),
            ("afraid", -0.70, 0.75, -0.30, "fear"),
            ("frightened", -0.80, 0.85, -0.40, "fear"),
            ("panicked", -0.88, 0.96, -0.60, "fear"),
            ("dread", -0.82, 0.78, -0.55, "fear"),
            ("nervous", -0.50, 0.65, -0.25, "fear"),
            ("uneasy", -0.40, 0.50, -0.20, "fear"),
            ("alarmed", -0.65, 0.80, -0.30, "fear"),
            ("horrified", -0.90, 0.94, -0.55, "fear"),
            ("petrified", -0.92, 0.96, -0.58, "fear"),
            ("paranoid", -0.55, 0.70, -0.35, "fear"),
            ("timid", -0.30, -0.20, -0.50, "fear"),
            ("shy", -0.20, -0.10, -0.40, "fear"),
            ("insecure", -0.40, 0.30, -0.55, "fear"),
            ("threatened", -0.70, 0.75, -0.45, "fear"),
            ("apprehensive", -0.55, 0.60, -0.30, "fear"),
            ("terrifying", -0.88, 0.93, -0.48, "fear"),
            ("creepy", -0.50, 0.60, -0.20, "fear"),
            ("spooky", -0.30, 0.55, -0.10, "fear"),
            ("daunting", -0.55, 0.50, -0.35, "fear"),
            ("intimidated", -0.55, 0.45, -0.50, "fear"),
            ("helpless", -0.70, 0.20, -0.75, "fear"),
            // === BASIC EMOTIONS — Joy ===
            ("joy", 0.90, 0.60, 0.70, "joy"),
            ("happy", 0.85, 0.50, 0.65, "joy"),
            ("delight", 0.88, 0.55, 0.68, "joy"),
            ("elated", 0.92, 0.70, 0.75, "joy"),
            ("ecstatic", 0.95, 0.80, 0.78, "joy"),
            ("joyful", 0.90, 0.60, 0.70, "joy"),
            ("cheerful", 0.85, 0.55, 0.60, "joy"),
            ("merry", 0.82, 0.50, 0.55, "joy"),
            ("gleeful", 0.88, 0.65, 0.62, "joy"),
            ("jubilant", 0.92, 0.75, 0.72, "joy"),
            ("thrilled", 0.90, 0.80, 0.70, "joy"),
            ("excited", 0.80, 0.85, 0.60, "joy"),
            ("euphoric", 0.93, 0.82, 0.76, "joy"),
            ("blissful", 0.95, 0.40, 0.65, "joy"),
            ("content", 0.70, -0.20, 0.50, "joy"),
            ("satisfied", 0.65, -0.10, 0.55, "joy"),
            ("pleased", 0.70, 0.20, 0.50, "joy"),
            ("amused", 0.65, 0.45, 0.40, "joy"),
            ("playful", 0.70, 0.55, 0.55, "joy"),
            ("upbeat", 0.78, 0.60, 0.58, "joy"),
            ("optimistic", 0.75, 0.40, 0.60, "joy"),
            ("hopeful", 0.70, 0.30, 0.40, "joy"),
            ("glad", 0.75, 0.35, 0.50, "joy"),
            ("festive", 0.80, 0.65, 0.55, "joy"),
            ("lively", 0.75, 0.70, 0.60, "joy"),
            ("exhilarated", 0.90, 0.85, 0.72, "joy"),
            ("radiant", 0.85, 0.50, 0.68, "joy"),
            ("sunny", 0.72, 0.35, 0.50, "joy"),
            ("exuberant", 0.88, 0.78, 0.70, "joy"),
            ("buoyant", 0.78, 0.55, 0.60, "joy"),
            // === BASIC EMOTIONS — Sadness ===
            ("sad", -0.70, -0.40, -0.30, "sadness"),
            ("sadness", -0.72, -0.38, -0.32, "sadness"),
            ("grief", -0.85, -0.30, -0.50, "sadness"),
            ("depressed", -0.90, -0.50, -0.60, "sadness"),
            ("melancholy", -0.65, -0.30, -0.35, "sadness"),
            ("sorrow", -0.80, -0.20, -0.45, "sadness"),
            ("heartbroken", -0.92, -0.10, -0.65, "sadness"),
            ("mournful", -0.82, -0.25, -0.50, "sadness"),
            ("miserable", -0.88, -0.30, -0.58, "sadness"),
            ("gloomy", -0.65, -0.40, -0.30, "sadness"),
            ("somber", -0.55, -0.45, -0.20, "sadness"),
            ("hopeless", -0.85, -0.20, -0.70, "sadness"),
            ("lonely", -0.70, -0.30, -0.55, "sadness"),
            ("devastated", -0.90, -0.10, -0.68, "sadness"),
            ("disheartened", -0.72, -0.25, -0.48, "sadness"),
            ("despondent", -0.80, -0.35, -0.55, "sadness"),
            ("downcast", -0.60, -0.40, -0.35, "sadness"),
            ("woeful", -0.75, -0.15, -0.42, "sadness"),
            ("wretched", -0.82, -0.20, -0.55, "sadness"),
            ("dismal", -0.68, -0.35, -0.30, "sadness"),
            ("dreary", -0.55, -0.40, -0.25, "sadness"),
            ("forlorn", -0.72, -0.30, -0.50, "sadness"),
            ("dejected", -0.70, -0.35, -0.45, "sadness"),
            ("crestfallen", -0.65, -0.20, -0.40, "sadness"),
            ("weepy", -0.60, -0.10, -0.35, "sadness"),
            ("tearful", -0.65, -0.15, -0.38, "sadness"),
            ("homesick", -0.55, -0.10, -0.30, "sadness"),
            ("bittersweet", -0.10, 0.10, 0.00, "sadness"),
            // === BASIC EMOTIONS — Disgust ===
            ("disgust", -0.75, 0.40, 0.10, "disgust"),
            ("revulsion", -0.80, 0.45, 0.05, "disgust"),
            ("repulsed", -0.78, 0.50, 0.08, "disgust"),
            ("appalled", -0.75, 0.60, -0.05, "disgust"),
            ("nauseated", -0.70, 0.30, -0.10, "disgust"),
            ("sickened", -0.72, 0.35, -0.08, "disgust"),
            ("grossed", -0.60, 0.40, 0.00, "disgust"),
            ("abhorrent", -0.82, 0.55, 0.15, "disgust"),
            ("loathsome", -0.80, 0.50, 0.12, "disgust"),
            ("detestable", -0.78, 0.48, 0.10, "disgust"),
            ("repulsive", -0.76, 0.45, 0.06, "disgust"),
            ("yucky", -0.50, 0.30, -0.05, "disgust"),
            ("icky", -0.45, 0.25, -0.02, "disgust"),
            ("vile", -0.80, 0.50, 0.15, "disgust"),
            ("despicable", -0.78, 0.52, 0.10, "disgust"),
            ("odious", -0.75, 0.45, 0.08, "disgust"),
            ("hideous", -0.70, 0.50, -0.05, "disgust"),
            ("ghastly", -0.72, 0.55, -0.10, "disgust"),
            ("putrid", -0.68, 0.30, -0.05, "disgust"),
            ("foul", -0.65, 0.35, 0.00, "disgust"),
            // === BASIC EMOTIONS — Surprise ===
            ("surprise", 0.30, 0.80, 0.20, "surprise"),
            ("shock", -0.20, 0.90, -0.10, "surprise"),
            ("astonished", 0.40, 0.85, 0.25, "surprise"),
            ("amazed", 0.60, 0.80, 0.35, "surprise"),
            ("startled", -0.10, 0.88, -0.05, "surprise"),
            ("stunned", -0.15, 0.85, -0.15, "surprise"),
            ("dumbfounded", 0.10, 0.82, 0.00, "surprise"),
            ("flabbergasted", 0.25, 0.86, 0.10, "surprise"),
            ("bewildered", -0.20, 0.70, -0.20, "surprise"),
            ("baffled", -0.10, 0.65, -0.15, "surprise"),
            ("perplexed", -0.15, 0.55, -0.20, "surprise"),
            ("speechless", 0.00, 0.75, -0.10, "surprise"),
            ("thunderstruck", 0.20, 0.90, 0.05, "surprise"),
            ("jawdropping", 0.60, 0.88, 0.30, "surprise"),
            ("unexpected", 0.10, 0.60, 0.10, "surprise"),
            ("remarkable", 0.60, 0.55, 0.40, "surprise"),
            ("extraordinary", 0.65, 0.60, 0.45, "surprise"),
            ("unprecedented", 0.30, 0.65, 0.25, "surprise"),
            // === BASIC EMOTIONS — Trust ===
            ("trust", 0.80, 0.20, 0.60, "trust"),
            ("betrayal", -0.75, 0.60, -0.40, "trust"),
            ("trusting", 0.75, 0.15, 0.55, "trust"),
            ("trustworthy", 0.80, 0.10, 0.65, "trust"),
            ("faith", 0.75, 0.15, 0.50, "trust"),
            ("faithful", 0.78, 0.12, 0.58, "trust"),
            ("loyalty", 0.80, 0.10, 0.62, "trust"),
            ("loyal", 0.78, 0.15, 0.60, "trust"),
            ("reliable", 0.70, 0.05, 0.55, "trust"),
            ("dependable", 0.68, 0.05, 0.52, "trust"),
            ("betrayed", -0.78, 0.58, -0.42, "trust"),
            ("deceived", -0.70, 0.55, -0.35, "trust"),
            ("misled", -0.55, 0.40, -0.30, "trust"),
            ("cheated", -0.65, 0.50, -0.25, "trust"),
            ("backstabbed", -0.80, 0.65, -0.45, "trust"),
            ("disloyal", -0.65, 0.30, -0.20, "trust"),
            ("unfaithful", -0.70, 0.35, -0.25, "trust"),
            ("sincere", 0.75, 0.10, 0.55, "trust"),
            ("honest", 0.78, 0.08, 0.60, "trust"),
            ("genuine", 0.72, 0.05, 0.50, "trust"),
            // === BASIC EMOTIONS — Anticipation ===
            ("anticipation", 0.50, 0.60, 0.35, "anticipation"),
            ("expectant", 0.40, 0.55, 0.30, "anticipation"),
            ("eager", 0.55, 0.70, 0.45, "anticipation"),
            ("longing", 0.30, 0.40, -0.10, "anticipation"),
            ("yearning", 0.35, 0.45, -0.05, "anticipation"),
            ("hopeful", 0.70, 0.30, 0.40, "anticipation"),
            ("desire", 0.60, 0.65, 0.50, "anticipation"),
            ("craving", 0.30, 0.70, 0.35, "anticipation"),
            ("awaiting", 0.20, 0.40, 0.20, "anticipation"),
            ("foreseeing", 0.30, 0.35, 0.40, "anticipation"),
            ("impatient", -0.20, 0.70, 0.25, "anticipation"),
            ("restless", -0.10, 0.65, 0.15, "anticipation"),
            ("aspiring", 0.65, 0.45, 0.60, "anticipation"),
            ("ambitious", 0.60, 0.55, 0.70, "anticipation"),
            ("driven", 0.55, 0.60, 0.65, "anticipation"),
            ("motivated", 0.65, 0.50, 0.60, "anticipation"),
            ("determined", 0.50, 0.55, 0.75, "anticipation"),
            ("looking forward", 0.60, 0.45, 0.40, "anticipation"),
            // === SERENITY / CALM ===
            ("calm", 0.60, -0.60, 0.30, "serenity"),
            ("peace", 0.70, -0.70, 0.40, "serenity"),
            ("serene", 0.75, -0.65, 0.45, "serenity"),
            ("tranquil", 0.72, -0.70, 0.42, "serenity"),
            ("relaxed", 0.65, -0.55, 0.35, "serenity"),
            ("peaceful", 0.72, -0.68, 0.40, "serenity"),
            ("composed", 0.55, -0.50, 0.55, "serenity"),
            ("collected", 0.50, -0.45, 0.50, "serenity"),
            ("placid", 0.55, -0.62, 0.35, "serenity"),
            ("restful", 0.60, -0.60, 0.30, "serenity"),
            ("mellow", 0.55, -0.40, 0.30, "serenity"),
            ("zen", 0.65, -0.70, 0.45, "serenity"),
            ("harmony", 0.72, -0.50, 0.50, "serenity"),
            ("balanced", 0.55, -0.30, 0.55, "serenity"),
            ("soothing", 0.68, -0.55, 0.38, "serenity"),
            ("gentle", 0.65, -0.40, 0.35, "serenity"),
            ("soft", 0.45, -0.45, 0.25, "serenity"),
            ("quiet", 0.40, -0.60, 0.20, "serenity"),
            ("still", 0.35, -0.65, 0.15, "serenity"),
            ("mindful", 0.60, -0.35, 0.50, "serenity"),
            // === SOCIAL EMOTIONS — Shame & Guilt ===
            ("shame", -0.70, 0.30, -0.50, "shame"),
            ("shameful", -0.75, 0.35, -0.55, "shame"),
            ("ashamed", -0.72, 0.32, -0.52, "shame"),
            ("embarrassed", -0.55, 0.50, -0.35, "shame"),
            ("humiliated", -0.80, 0.60, -0.60, "shame"),
            ("mortified", -0.78, 0.65, -0.55, "shame"),
            ("disgraced", -0.82, 0.50, -0.58, "shame"),
            ("dishonor", -0.75, 0.40, -0.50, "shame"),
            ("remorse", -0.65, 0.20, -0.40, "shame"),
            ("regretful", -0.60, 0.25, -0.35, "shame"),
            ("contrite", -0.55, 0.15, -0.38, "shame"),
            ("apologetic", -0.40, 0.20, -0.30, "shame"),
            ("guilt", -0.65, 0.25, -0.45, "guilt"),
            ("guilty", -0.68, 0.28, -0.48, "guilt"),
            ("culpable", -0.70, 0.30, -0.42, "guilt"),
            ("blameworthy", -0.72, 0.32, -0.44, "guilt"),
            ("responsible", 0.10, 0.15, 0.55, "guilt"),
            ("accountable", 0.05, 0.20, 0.50, "guilt"),
            // === SOCIAL EMOTIONS — Pride ===
            ("pride", 0.70, 0.40, 0.75, "pride"),
            ("proud", 0.75, 0.38, 0.78, "pride"),
            ("accomplished", 0.78, 0.35, 0.80, "pride"),
            ("dignified", 0.65, 0.10, 0.72, "pride"),
            ("dignity", 0.68, 0.08, 0.70, "pride"),
            ("selfrespect", 0.72, 0.05, 0.75, "pride"),
            ("honor", 0.75, 0.20, 0.78, "pride"),
            ("glory", 0.82, 0.65, 0.80, "pride"),
            ("haughty", -0.30, 0.35, 0.50, "pride"),
            ("arrogant", -0.45, 0.40, 0.65, "pride"),
            ("conceited", -0.40, 0.30, 0.60, "pride"),
            ("smug", -0.20, 0.25, 0.50, "pride"),
            ("vain", -0.35, 0.20, 0.45, "pride"),
            ("narcissistic", -0.55, 0.30, 0.60, "pride"),
            ("egotistical", -0.50, 0.35, 0.62, "pride"),
            ("boastful", -0.30, 0.50, 0.55, "pride"),
            // === SOCIAL EMOTIONS — Envy & Jealousy ===
            ("envy", -0.45, 0.40, 0.05, "envy"),
            ("envious", -0.48, 0.42, 0.02, "envy"),
            ("jealous", -0.55, 0.50, -0.05, "envy"),
            ("jealousy", -0.52, 0.48, -0.02, "envy"),
            ("covetous", -0.50, 0.45, 0.08, "envy"),
            ("resentful", -0.65, 0.40, 0.00, "envy"),
            ("grudging", -0.50, 0.30, 0.05, "envy"),
            ("greeneyed", -0.55, 0.45, 0.00, "envy"),
            ("possessive", -0.30, 0.50, 0.20, "jealousy"),
            ("overprotective", -0.20, 0.55, 0.15, "jealousy"),
            ("insecure", -0.40, 0.30, -0.55, "jealousy"),
            // === SOCIAL EMOTIONS — Gratitude ===
            ("gratitude", 0.85, 0.20, 0.40, "gratitude"),
            ("grateful", 0.85, 0.22, 0.42, "gratitude"),
            ("thankful", 0.80, 0.18, 0.38, "gratitude"),
            ("appreciative", 0.78, 0.20, 0.40, "gratitude"),
            ("blessed", 0.82, 0.15, 0.45, "gratitude"),
            ("indebted", 0.50, 0.10, 0.15, "gratitude"),
            ("obliged", 0.30, 0.05, 0.10, "gratitude"),
            ("thankful", 0.80, 0.18, 0.38, "gratitude"),
            // === SOCIAL EMOTIONS — Admiration ===
            ("admiration", 0.80, 0.35, 0.50, "admiration"),
            ("admire", 0.78, 0.32, 0.48, "admiration"),
            ("adore", 0.85, 0.45, 0.40, "admiration"),
            ("adoration", 0.88, 0.40, 0.42, "admiration"),
            ("worship", 0.75, 0.50, 0.30, "admiration"),
            ("revere", 0.72, 0.30, 0.45, "admiration"),
            ("reverence", 0.70, 0.25, 0.50, "admiration"),
            ("venerate", 0.68, 0.28, 0.48, "admiration"),
            ("respect", 0.72, 0.10, 0.60, "admiration"),
            ("esteem", 0.70, 0.12, 0.55, "admiration"),
            ("look up", 0.65, 0.20, 0.40, "admiration"),
            ("idolize", 0.70, 0.55, 0.35, "admiration"),
            ("hero", 0.80, 0.60, 0.70, "admiration"),
            ("heroic", 0.82, 0.62, 0.72, "admiration"),
            ("legendary", 0.80, 0.55, 0.75, "admiration"),
            // === SOCIAL EMOTIONS — Contempt ===
            ("contempt", -0.60, 0.50, 0.30, "contempt"),
            ("disdain", -0.65, 0.45, 0.35, "contempt"),
            ("scorn", -0.70, 0.55, 0.38, "contempt"),
            ("disrespect", -0.55, 0.40, 0.25, "contempt"),
            ("sneer", -0.50, 0.35, 0.30, "contempt"),
            ("belittle", -0.58, 0.30, 0.40, "contempt"),
            ("mock", -0.45, 0.50, 0.35, "contempt"),
            ("ridicule", -0.50, 0.55, 0.30, "contempt"),
            ("derision", -0.55, 0.48, 0.32, "contempt"),
            ("condescending", -0.60, 0.25, 0.55, "contempt"),
            ("patronizing", -0.55, 0.20, 0.50, "contempt"),
            ("superior", -0.20, 0.15, 0.75, "contempt"),
            // === SOCIAL EMOTIONS — Sympathy ===
            ("sympathy", 0.50, 0.15, 0.10, "sympathy"),
            ("sympathetic", 0.55, 0.12, 0.15, "sympathy"),
            ("compassion", 0.72, 0.10, 0.35, "sympathy"),
            ("compassionate", 0.75, 0.12, 0.38, "sympathy"),
            ("empathy", 0.60, 0.15, 0.25, "sympathy"),
            ("empathetic", 0.62, 0.18, 0.28, "sympathy"),
            ("pity", -0.30, 0.10, -0.20, "sympathy"),
            ("pitiful", -0.50, 0.20, -0.30, "sympathy"),
            ("merciful", 0.68, 0.05, 0.45, "sympathy"),
            ("mercy", 0.65, 0.05, 0.42, "sympathy"),
            ("tender", 0.60, 0.10, 0.15, "sympathy"),
            ("tenderness", 0.62, 0.12, 0.18, "sympathy"),
            ("warmth", 0.72, 0.15, 0.40, "sympathy"),
            ("caring", 0.75, 0.10, 0.38, "sympathy"),
            ("kindness", 0.78, 0.08, 0.45, "sympathy"),
            ("kind", 0.72, 0.05, 0.42, "sympathy"),
            ("thoughtful", 0.65, 0.05, 0.40, "sympathy"),
            ("considerate", 0.68, 0.02, 0.42, "sympathy"),
            // === COGNITIVE STATES — Confusion ===
            ("confused", -0.30, 0.50, -0.30, "confusion"),
            ("confusion", -0.25, 0.48, -0.28, "confusion"),
            ("bewildered", -0.20, 0.70, -0.20, "confusion"),
            ("baffled", -0.10, 0.65, -0.15, "confusion"),
            ("perplexed", -0.15, 0.55, -0.20, "confusion"),
            ("puzzled", -0.10, 0.40, -0.15, "confusion"),
            ("mystified", -0.10, 0.50, -0.10, "confusion"),
            ("disoriented", -0.35, 0.55, -0.40, "confusion"),
            ("lost", -0.45, 0.20, -0.55, "confusion"),
            ("uncertain", -0.30, 0.35, -0.35, "confusion"),
            ("unsure", -0.25, 0.30, -0.30, "confusion"),
            ("ambiguous", -0.10, 0.15, -0.15, "confusion"),
            ("vague", -0.15, 0.10, -0.20, "confusion"),
            ("fuzzy", -0.10, 0.05, -0.15, "confusion"),
            ("unclear", -0.20, 0.20, -0.25, "confusion"),
            ("mixed", -0.05, 0.20, -0.10, "confusion"),
            ("torn", -0.30, 0.45, -0.25, "confusion"),
            ("conflicted", -0.35, 0.50, -0.20, "confusion"),
            // === COGNITIVE STATES — Curiosity & Interest ===
            ("curious", 0.50, 0.55, 0.35, "curiosity"),
            ("curiosity", 0.52, 0.58, 0.32, "curiosity"),
            ("interested", 0.55, 0.45, 0.40, "interest"),
            ("interest", 0.50, 0.40, 0.38, "interest"),
            ("intrigued", 0.60, 0.55, 0.40, "interest"),
            ("fascinated", 0.70, 0.65, 0.45, "interest"),
            ("captivated", 0.72, 0.60, 0.42, "interest"),
            ("engaged", 0.55, 0.50, 0.45, "interest"),
            ("absorbed", 0.50, 0.35, 0.40, "interest"),
            ("attentive", 0.45, 0.40, 0.50, "interest"),
            ("focused", 0.40, 0.45, 0.65, "interest"),
            ("inquisitive", 0.55, 0.55, 0.40, "curiosity"),
            ("questioning", 0.10, 0.40, 0.15, "curiosity"),
            ("skeptical", -0.10, 0.35, 0.20, "curiosity"),
            ("doubtful", -0.25, 0.30, -0.10, "curiosity"),
            ("probing", 0.20, 0.45, 0.35, "curiosity"),
            ("exploring", 0.50, 0.55, 0.45, "curiosity"),
            ("wondering", 0.30, 0.35, 0.10, "curiosity"),
            ("inspired", 0.80, 0.65, 0.65, "interest"),
            ("motivated", 0.65, 0.50, 0.60, "interest"),
            // === COGNITIVE STATES — Boredom ===
            ("bored", -0.50, -0.60, -0.20, "boredom"),
            ("boredom", -0.52, -0.62, -0.22, "boredom"),
            ("tedious", -0.45, -0.50, -0.15, "boredom"),
            ("monotonous", -0.40, -0.55, -0.10, "boredom"),
            ("dull", -0.35, -0.55, -0.15, "boredom"),
            ("mundane", -0.30, -0.45, -0.10, "boredom"),
            ("tiresome", -0.45, -0.30, -0.20, "boredom"),
            ("repetitive", -0.30, -0.35, -0.10, "boredom"),
            ("uninteresting", -0.45, -0.50, -0.15, "boredom"),
            ("uninspired", -0.50, -0.40, -0.30, "boredom"),
            ("apathetic", -0.40, -0.55, -0.35, "boredom"),
            ("indifferent", -0.25, -0.45, -0.15, "boredom"),
            ("detached", -0.20, -0.40, 0.10, "boredom"),
            ("disengaged", -0.35, -0.45, -0.20, "boredom"),
            ("listless", -0.45, -0.50, -0.40, "boredom"),
            ("languid", -0.30, -0.55, -0.25, "boredom"),
            // === COGNITIVE STATES — Doubt & Certainty ===
            ("doubt", -0.30, 0.25, -0.25, "doubt"),
            ("doubtful", -0.35, 0.28, -0.30, "doubt"),
            ("uncertain", -0.30, 0.35, -0.35, "doubt"),
            ("skeptical", -0.10, 0.35, 0.20, "doubt"),
            ("suspicious", -0.45, 0.50, -0.10, "doubt"),
            ("distrustful", -0.55, 0.40, -0.15, "doubt"),
            ("wary", -0.35, 0.45, -0.20, "doubt"),
            ("cautious", -0.10, 0.25, 0.10, "doubt"),
            ("hesitant", -0.25, 0.30, -0.20, "doubt"),
            ("uncertainty", -0.28, 0.32, -0.32, "doubt"),
            ("ambivalent", -0.10, 0.20, -0.10, "doubt"),
            ("conflicted", -0.35, 0.50, -0.20, "doubt"),
            ("certain", 0.55, 0.20, 0.75, "certainty"),
            ("certainty", 0.55, 0.15, 0.78, "certainty"),
            ("confident", 0.65, 0.35, 0.80, "certainty"),
            ("sure", 0.50, 0.15, 0.70, "certainty"),
            ("definite", 0.50, 0.20, 0.72, "certainty"),
            ("convinced", 0.45, 0.25, 0.65, "certainty"),
            ("positive", 0.60, 0.30, 0.68, "certainty"),
            ("assured", 0.55, 0.10, 0.70, "certainty"),
            ("decided", 0.40, 0.15, 0.72, "certainty"),
            ("resolute", 0.50, 0.30, 0.78, "certainty"),
            ("unwavering", 0.55, 0.15, 0.82, "certainty"),
            ("insistent", 0.10, 0.55, 0.60, "certainty"),
            ("adamant", 0.15, 0.50, 0.62, "certainty"),
            // === COGNITIVE STATES — Insight ===
            ("insight", 0.70, 0.30, 0.60, "insight"),
            ("insightful", 0.72, 0.32, 0.62, "insight"),
            ("realization", 0.55, 0.50, 0.45, "insight"),
            ("epiphany", 0.75, 0.70, 0.50, "insight"),
            ("revelation", 0.60, 0.72, 0.40, "insight"),
            ("understanding", 0.60, 0.10, 0.55, "insight"),
            ("comprehension", 0.55, 0.05, 0.50, "insight"),
            ("awareness", 0.50, 0.10, 0.50, "insight"),
            ("clarity", 0.65, 0.05, 0.60, "insight"),
            ("discovery", 0.72, 0.65, 0.55, "insight"),
            ("breakthrough", 0.78, 0.75, 0.65, "insight"),
            ("enlightenment", 0.80, 0.40, 0.60, "insight"),
            ("wisdom", 0.75, 0.05, 0.70, "insight"),
            ("discernment", 0.60, 0.10, 0.60, "insight"),
            ("intuition", 0.50, 0.20, 0.45, "insight"),
            // === MORAL EMOTIONS ===
            ("outrage", -0.88, 0.92, 0.55, "anger"),
            ("outraged", -0.90, 0.93, 0.56, "anger"),
            ("indignation", -0.75, 0.78, 0.45, "anger"),
            ("indignant", -0.77, 0.80, 0.48, "anger"),
            ("moral outrage", -0.85, 0.88, 0.50, "anger"),
            ("righteous", 0.40, 0.55, 0.65, "anger"),
            ("appalled", -0.75, 0.60, -0.05, "disgust"),
            ("shocked", -0.20, 0.90, -0.10, "surprise"),
            ("compassion", 0.72, 0.10, 0.35, "compassion"),
            ("compassionate", 0.75, 0.12, 0.38, "compassion"),
            ("elevation", 0.80, 0.35, 0.55, "elevation"),
            ("elevated", 0.78, 0.30, 0.55, "elevation"),
            ("uplifted", 0.82, 0.40, 0.52, "elevation"),
            ("inspired", 0.80, 0.65, 0.65, "elevation"),
            ("noble", 0.75, 0.20, 0.70, "elevation"),
            ("virtuous", 0.78, 0.10, 0.72, "elevation"),
            ("schadenfreude", -0.20, 0.55, 0.25, "schadenfreude"),
            ("gloating", -0.30, 0.60, 0.30, "schadenfreude"),
            ("malicious joy", -0.35, 0.58, 0.28, "schadenfreude"),
            ("smug", -0.20, 0.25, 0.50, "schadenfreude"),
            // === DISINFORMATION SIGNALS ===
            ("conspiracy", -0.50, 0.60, -0.10, "distrust"),
            ("conspiratorial", -0.55, 0.65, -0.15, "distrust"),
            ("conspire", -0.60, 0.55, -0.10, "distrust"),
            ("coverup", -0.65, 0.58, -0.20, "distrust"),
            ("cover-up", -0.62, 0.56, -0.18, "distrust"),
            ("whistleblower", 0.30, 0.70, 0.35, "surprise"),
            ("whistle-blower", 0.30, 0.70, 0.35, "surprise"),
            ("leaked", -0.20, 0.75, 0.10, "surprise"),
            ("leak", -0.15, 0.70, 0.05, "surprise"),
            ("exposed", -0.30, 0.75, 0.10, "surprise"),
            ("expose", -0.25, 0.72, 0.12, "surprise"),
            ("scandal", -0.65, 0.70, 0.00, "disgust"),
            ("scandalous", -0.68, 0.72, -0.02, "disgust"),
            ("rigged", -0.70, 0.60, -0.15, "anger"),
            ("rigging", -0.68, 0.58, -0.12, "anger"),
            ("fraudulent", -0.72, 0.50, -0.10, "anger"),
            ("fraud", -0.70, 0.55, -0.08, "anger"),
            ("deceptive", -0.65, 0.45, -0.05, "distrust"),
            ("deception", -0.68, 0.48, -0.08, "distrust"),
            ("misleading", -0.55, 0.40, -0.10, "distrust"),
            ("mislead", -0.55, 0.42, -0.12, "distrust"),
            ("dishonest", -0.62, 0.35, -0.05, "distrust"),
            ("dishonesty", -0.60, 0.38, -0.08, "distrust"),
            ("corrupt", -0.72, 0.52, -0.15, "anger"),
            ("corruption", -0.75, 0.50, -0.20, "anger"),
            ("collusion", -0.60, 0.45, -0.15, "distrust"),
            ("collude", -0.58, 0.48, -0.12, "distrust"),
            ("manipulate", -0.55, 0.50, 0.20, "distrust"),
            ("manipulation", -0.58, 0.52, 0.18, "distrust"),
            ("propaganda", -0.50, 0.45, -0.05, "distrust"),
            ("censorship", -0.55, 0.35, -0.30, "anger"),
            ("censor", -0.50, 0.38, -0.25, "anger"),
            ("suppressed", -0.60, 0.30, -0.40, "anger"),
            ("suppression", -0.58, 0.32, -0.38, "anger"),
            ("disinformation", -0.55, 0.48, -0.10, "distrust"),
            ("misinformation", -0.45, 0.35, -0.08, "distrust"),
            ("fabricated", -0.50, 0.40, -0.05, "distrust"),
            ("fabrication", -0.52, 0.42, -0.08, "distrust"),
            ("hoax", -0.45, 0.65, 0.05, "distrust"),
            ("fake", -0.40, 0.45, 0.00, "distrust"),
            ("phony", -0.48, 0.40, -0.02, "distrust"),
            ("lies", -0.60, 0.40, -0.10, "distrust"),
            ("lie", -0.55, 0.38, -0.05, "distrust"),
            ("liar", -0.65, 0.42, 0.05, "distrust"),
            ("coverup", -0.65, 0.58, -0.20, "distrust"),
            ("cover-up", -0.62, 0.56, -0.18, "distrust"),
            ("unravel", -0.20, 0.55, 0.10, "surprise"),
            ("uncovered", -0.10, 0.65, 0.15, "surprise"),
            ("exposé", -0.20, 0.72, 0.12, "surprise"),
            // === LOVE & AFFECTION ===
            ("love", 0.95, 0.60, 0.50, "love"),
            ("loved", 0.93, 0.55, 0.48, "love"),
            ("beloved", 0.95, 0.45, 0.45, "love"),
            ("cherished", 0.90, 0.30, 0.55, "love"),
            ("cherish", 0.88, 0.35, 0.52, "love"),
            ("affection", 0.85, 0.30, 0.45, "love"),
            ("affectionate", 0.88, 0.32, 0.48, "love"),
            ("devotion", 0.85, 0.25, 0.50, "love"),
            ("devoted", 0.82, 0.28, 0.52, "love"),
            ("passion", 0.80, 0.85, 0.60, "love"),
            ("passionate", 0.82, 0.88, 0.62, "love"),
            ("romance", 0.85, 0.60, 0.45, "love"),
            ("romantic", 0.82, 0.55, 0.42, "love"),
            ("intimate", 0.75, 0.45, 0.35, "love"),
            ("intimacy", 0.78, 0.40, 0.38, "love"),
            ("tender", 0.60, 0.10, 0.15, "love"),
            ("tenderness", 0.62, 0.12, 0.18, "love"),
            ("fond", 0.75, 0.20, 0.40, "love"),
            ("fondness", 0.78, 0.22, 0.38, "love"),
            ("warm", 0.72, 0.20, 0.42, "love"),
            ("warmth", 0.72, 0.15, 0.40, "love"),
            ("caring", 0.75, 0.10, 0.38, "love"),
            ("care", 0.65, 0.10, 0.35, "love"),
            ("adore", 0.85, 0.45, 0.40, "love"),
            ("adoration", 0.88, 0.40, 0.42, "love"),
            // === HATE & HOSTILITY ===
            ("hate", -0.90, 0.85, 0.50, "anger"),
            ("hatred", -0.92, 0.88, 0.52, "anger"),
            ("loathe", -0.88, 0.80, 0.48, "anger"),
            ("loathing", -0.85, 0.78, 0.45, "anger"),
            ("detest", -0.82, 0.75, 0.42, "anger"),
            ("despise", -0.85, 0.78, 0.44, "anger"),
            ("abhor", -0.80, 0.72, 0.40, "anger"),
            ("hostility", -0.72, 0.75, 0.48, "anger"),
            ("hostile", -0.75, 0.80, 0.55, "anger"),
            ("aggressive", -0.60, 0.78, 0.60, "anger"),
            ("violence", -0.85, 0.85, 0.30, "anger"),
            ("violent", -0.82, 0.88, 0.35, "anger"),
            ("cruel", -0.80, 0.60, 0.40, "anger"),
            ("cruelty", -0.82, 0.62, 0.38, "anger"),
            ("savage", -0.75, 0.72, 0.50, "anger"),
            ("brutal", -0.80, 0.75, 0.45, "anger"),
            ("brutality", -0.82, 0.78, 0.42, "anger"),
            ("vicious", -0.78, 0.70, 0.48, "anger"),
            ("malice", -0.75, 0.55, 0.35, "anger"),
            ("malicious", -0.72, 0.58, 0.38, "anger"),
            ("spite", -0.60, 0.45, 0.25, "anger"),
            ("spiteful", -0.62, 0.48, 0.28, "anger"),
            ("vengeful", -0.68, 0.60, 0.40, "anger"),
            ("revenge", -0.60, 0.75, 0.45, "anger"),
            ("vindictive", -0.65, 0.62, 0.42, "anger"),
            // === STRESS & OVERWHELM ===
            ("stressed", -0.55, 0.75, -0.30, "stress"),
            ("stress", -0.50, 0.72, -0.28, "stress"),
            ("overwhelmed", -0.60, 0.80, -0.45, "stress"),
            ("overwhelming", -0.55, 0.82, -0.40, "stress"),
            ("exhausted", -0.55, -0.30, -0.45, "stress"),
            ("burnout", -0.60, -0.20, -0.50, "stress"),
            ("burned out", -0.62, -0.15, -0.52, "stress"),
            ("drained", -0.55, -0.35, -0.45, "stress"),
            ("depleted", -0.50, -0.30, -0.40, "stress"),
            ("fatigued", -0.45, -0.40, -0.35, "stress"),
            ("tired", -0.35, -0.45, -0.25, "stress"),
            ("weary", -0.45, -0.30, -0.35, "stress"),
            ("worn out", -0.50, -0.25, -0.40, "stress"),
            ("pressured", -0.40, 0.65, -0.15, "stress"),
            ("under pressure", -0.40, 0.68, -0.18, "stress"),
            ("burdened", -0.50, 0.30, -0.35, "stress"),
            ("crushed", -0.78, 0.20, -0.65, "stress"),
            ("suffocating", -0.70, 0.50, -0.55, "stress"),
            ("drowning", -0.65, 0.55, -0.55, "stress"),
            ("panicking", -0.85, 0.95, -0.58, "stress"),
            ("frazzled", -0.45, 0.70, -0.20, "stress"),
            ("distracted", -0.25, 0.35, -0.20, "stress"),
            // === SURPRISE VARIETIES — Amazement ===
            ("amazing", 0.80, 0.75, 0.55, "surprise"),
            ("amazed", 0.60, 0.80, 0.35, "surprise"),
            ("astonishing", 0.65, 0.82, 0.40, "surprise"),
            ("astounding", 0.70, 0.85, 0.42, "surprise"),
            ("staggering", 0.45, 0.78, 0.30, "surprise"),
            ("mindblowing", 0.75, 0.88, 0.50, "surprise"),
            ("breathtaking", 0.78, 0.80, 0.48, "surprise"),
            ("spectacular", 0.82, 0.75, 0.58, "surprise"),
            ("magnificent", 0.85, 0.65, 0.65, "surprise"),
            ("splendid", 0.80, 0.55, 0.60, "surprise"),
            ("wonderful", 0.88, 0.55, 0.62, "surprise"),
            ("marvelous", 0.85, 0.60, 0.58, "surprise"),
            ("fantastic", 0.85, 0.68, 0.60, "surprise"),
            ("incredible", 0.78, 0.78, 0.52, "surprise"),
            ("unbelievable", 0.50, 0.85, 0.30, "surprise"),
            ("phenomenal", 0.82, 0.72, 0.62, "surprise"),
            ("extraordinary", 0.65, 0.60, 0.45, "surprise"),
            ("remarkable", 0.60, 0.55, 0.40, "surprise"),
            ("impressive", 0.75, 0.50, 0.55, "surprise"),
            ("stunning", 0.50, 0.78, 0.35, "surprise"),
            // === REGRET & NOSTALGIA ===
            ("regret", -0.60, 0.20, -0.40, "regret"),
            ("regretful", -0.62, 0.22, -0.42, "regret"),
            ("remorse", -0.65, 0.20, -0.40, "regret"),
            ("remorseful", -0.68, 0.18, -0.42, "regret"),
            ("repentant", -0.55, 0.15, -0.35, "regret"),
            ("penitent", -0.50, 0.12, -0.38, "regret"),
            ("nostalgic", 0.30, 0.10, 0.05, "nostalgia"),
            ("nostalgia", 0.35, 0.12, 0.08, "nostalgia"),
            ("wistful", 0.20, 0.15, 0.00, "nostalgia"),
            ("sentimental", 0.40, 0.20, 0.10, "nostalgia"),
            ("longing", 0.30, 0.40, -0.10, "nostalgia"),
            ("yearning", 0.35, 0.45, -0.05, "nostalgia"),
            ("reminisce", 0.45, 0.15, 0.20, "nostalgia"),
            ("reminiscent", 0.40, 0.10, 0.15, "nostalgia"),
            ("remembering", 0.30, 0.05, 0.10, "nostalgia"),
            // === INTERPERSONAL — Rejection & Abandonment ===
            ("rejected", -0.72, 0.40, -0.50, "rejection"),
            ("rejection", -0.70, 0.38, -0.48, "rejection"),
            ("abandoned", -0.80, 0.30, -0.62, "rejection"),
            ("abandonment", -0.78, 0.28, -0.60, "rejection"),
            ("forsaken", -0.82, 0.25, -0.65, "rejection"),
            ("deserted", -0.78, 0.35, -0.58, "rejection"),
            ("isolated", -0.60, 0.10, -0.55, "rejection"),
            ("excluded", -0.55, 0.30, -0.45, "rejection"),
            ("left out", -0.55, 0.25, -0.40, "rejection"),
            ("ostracized", -0.70, 0.35, -0.55, "rejection"),
            ("shunned", -0.72, 0.32, -0.52, "rejection"),
            ("ignored", -0.45, 0.20, -0.40, "rejection"),
            ("neglected", -0.55, 0.15, -0.48, "rejection"),
            ("unwanted", -0.60, 0.20, -0.50, "rejection"),
            ("unloved", -0.75, 0.10, -0.58, "rejection"),
            ("unappreciated", -0.55, 0.15, -0.35, "rejection"),
            // === INTERPERSONAL — Acceptance & Belonging ===
            ("accepted", 0.70, 0.15, 0.55, "acceptance"),
            ("acceptance", 0.65, 0.10, 0.50, "acceptance"),
            ("included", 0.65, 0.20, 0.45, "acceptance"),
            ("welcome", 0.75, 0.35, 0.50, "acceptance"),
            ("welcomed", 0.78, 0.32, 0.52, "acceptance"),
            ("belonging", 0.72, 0.15, 0.50, "acceptance"),
            ("embraced", 0.80, 0.30, 0.48, "acceptance"),
            ("embrace", 0.75, 0.35, 0.45, "acceptance"),
            ("connected", 0.65, 0.20, 0.45, "acceptance"),
            ("connection", 0.62, 0.25, 0.40, "acceptance"),
            ("community", 0.68, 0.15, 0.48, "acceptance"),
            ("togetherness", 0.75, 0.20, 0.50, "acceptance"),
            ("unity", 0.70, 0.15, 0.52, "acceptance"),
            ("solidarity", 0.68, 0.20, 0.55, "acceptance"),
            ("supported", 0.72, 0.10, 0.45, "acceptance"),
            ("support", 0.65, 0.10, 0.42, "acceptance"),
            // === SUFFERING & PAIN ===
            ("pain", -0.75, 0.40, -0.45, "suffering"),
            ("painful", -0.78, 0.42, -0.48, "suffering"),
            ("suffering", -0.80, 0.30, -0.55, "suffering"),
            ("suffer", -0.78, 0.35, -0.52, "suffering"),
            ("agony", -0.88, 0.50, -0.60, "suffering"),
            ("anguish", -0.85, 0.45, -0.58, "suffering"),
            ("torment", -0.82, 0.55, -0.55, "suffering"),
            ("tortured", -0.85, 0.60, -0.58, "suffering"),
            ("torture", -0.88, 0.65, -0.60, "suffering"),
            ("distress", -0.65, 0.60, -0.40, "suffering"),
            ("trauma", -0.80, 0.55, -0.60, "suffering"),
            ("traumatic", -0.82, 0.60, -0.62, "suffering"),
            ("hurt", -0.65, 0.35, -0.35, "suffering"),
            ("wounded", -0.60, 0.20, -0.40, "suffering"),
            ("injured", -0.50, 0.25, -0.30, "suffering"),
            ("damaged", -0.50, 0.15, -0.35, "suffering"),
            ("broken", -0.70, 0.10, -0.55, "suffering"),
            ("heartache", -0.78, 0.15, -0.50, "suffering"),
            ("heartbreak", -0.85, 0.20, -0.58, "suffering"),
            // === MISC — Negative ===
            ("terrible", -0.75, 0.50, -0.20, "anger"),
            ("awful", -0.72, 0.55, -0.15, "anger"),
            ("horrible", -0.78, 0.60, -0.18, "anger"),
            ("dreadful", -0.80, 0.58, -0.22, "fear"),
            ("atrocious", -0.82, 0.65, -0.10, "anger"),
            ("appalling", -0.78, 0.68, -0.05, "anger"),
            ("abysmal", -0.75, 0.30, -0.25, "sadness"),
            ("miserable", -0.88, -0.30, -0.58, "sadness"),
            ("wretched", -0.82, -0.20, -0.55, "sadness"),
            ("lousy", -0.55, 0.15, -0.20, "anger"),
            ("rotten", -0.50, 0.20, -0.10, "disgust"),
            ("nasty", -0.60, 0.55, 0.15, "disgust"),
            ("mean", -0.55, 0.35, 0.30, "anger"),
            ("wicked", -0.50, 0.45, 0.35, "anger"),
            ("evil", -0.78, 0.50, 0.40, "anger"),
            ("sinister", -0.70, 0.55, 0.30, "fear"),
            ("malevolent", -0.75, 0.50, 0.45, "anger"),
            ("malignant", -0.72, 0.40, 0.30, "anger"),
            ("perilous", -0.65, 0.60, -0.10, "fear"),
            ("dangerous", -0.60, 0.70, -0.05, "fear"),
            ("threatening", -0.65, 0.72, 0.10, "fear"),
            ("ominous", -0.60, 0.65, -0.05, "fear"),
            ("foreboding", -0.55, 0.60, -0.15, "fear"),
            ("unsettling", -0.45, 0.55, -0.15, "fear"),
            ("disturbing", -0.55, 0.60, -0.10, "fear"),
            ("troubling", -0.50, 0.50, -0.15, "fear"),
            ("alarming", -0.60, 0.75, -0.10, "fear"),
            ("shocking", -0.20, 0.90, -0.10, "surprise"),
            // === MISC — Positive ===
            ("beautiful", 0.85, 0.35, 0.60, "joy"),
            ("lovely", 0.82, 0.30, 0.50, "joy"),
            ("gorgeous", 0.85, 0.40, 0.55, "joy"),
            ("magnificent", 0.85, 0.65, 0.65, "joy"),
            ("glorious", 0.88, 0.60, 0.70, "joy"),
            ("splendid", 0.80, 0.55, 0.60, "joy"),
            ("excellent", 0.80, 0.40, 0.70, "joy"),
            ("great", 0.70, 0.35, 0.55, "joy"),
            ("good", 0.65, 0.15, 0.50, "joy"),
            ("nice", 0.60, 0.15, 0.40, "joy"),
            ("pleasant", 0.68, 0.10, 0.45, "joy"),
            ("positive", 0.60, 0.30, 0.55, "joy"),
            ("superb", 0.85, 0.50, 0.72, "joy"),
            ("outstanding", 0.82, 0.55, 0.75, "joy"),
            ("exceptional", 0.80, 0.50, 0.72, "joy"),
            ("fabulous", 0.85, 0.65, 0.60, "joy"),
            ("terrific", 0.82, 0.60, 0.65, "joy"),
            ("wonderful", 0.88, 0.55, 0.62, "joy"),
            ("marvelous", 0.85, 0.60, 0.58, "joy"),
            ("delightful", 0.88, 0.55, 0.68, "joy"),
            ("pleasurable", 0.80, 0.40, 0.55, "joy"),
            ("enjoyable", 0.75, 0.35, 0.50, "joy"),
            ("fun", 0.72, 0.60, 0.55, "joy"),
            ("awesome", 0.78, 0.68, 0.58, "joy"),
            ("amazing", 0.80, 0.75, 0.55, "joy"),
            // === AWE ===
            ("awe", 0.70, 0.65, 0.30, "awe"),
            ("awestruck", 0.65, 0.75, 0.25, "awe"),
            ("inspired", 0.80, 0.65, 0.65, "awe"),
            ("wonder", 0.72, 0.55, 0.40, "awe"),
            ("wonderment", 0.70, 0.50, 0.38, "awe"),
            ("transfixed", 0.40, 0.60, 0.15, "awe"),
            ("mesmerized", 0.65, 0.55, 0.30, "awe"),
            ("enchanted", 0.80, 0.45, 0.40, "awe"),
            ("magical", 0.82, 0.55, 0.45, "awe"),
            ("majestic", 0.85, 0.45, 0.65, "awe"),
            ("sublime", 0.80, 0.40, 0.55, "awe"),
            ("transcendent", 0.85, 0.45, 0.60, "awe"),
            ("miraculous", 0.88, 0.72, 0.50, "awe"),
            // === RELIEF ===
            ("relief", 0.70, -0.50, 0.40, "relief"),
            ("relieved", 0.72, -0.48, 0.42, "relief"),
            ("reassured", 0.65, -0.40, 0.50, "relief"),
            ("comforted", 0.68, -0.35, 0.45, "relief"),
            ("soothed", 0.65, -0.55, 0.35, "relief"),
            ("ease", 0.55, -0.45, 0.35, "relief"),
            ("peaceful", 0.72, -0.68, 0.40, "relief"),
            ("free", 0.78, 0.30, 0.72, "relief"),
            ("liberated", 0.80, 0.40, 0.75, "relief"),
            ("unburdened", 0.72, -0.20, 0.50, "relief"),
            ("lightened", 0.65, -0.15, 0.42, "relief"),
            // === SURPRISE NEGATIVE ===
            ("dismay", -0.60, 0.55, -0.25, "surprise"),
            ("dismayed", -0.62, 0.58, -0.28, "surprise"),
            ("alarm", -0.55, 0.72, -0.15, "surprise"),
            ("consternation", -0.65, 0.60, -0.20, "surprise"),
            ("chagrin", -0.50, 0.45, -0.18, "surprise"),
            ("disillusioned", -0.55, 0.25, -0.30, "surprise"),
            ("disenchanted", -0.50, 0.20, -0.25, "surprise"),
            ("taken aback", -0.20, 0.70, -0.10, "surprise"),
            ("caught off guard", -0.10, 0.72, -0.08, "surprise"),
            // === EMPOWERMENT ===
            ("empowered", 0.80, 0.55, 0.82, "empowerment"),
            ("empowerment", 0.78, 0.50, 0.80, "empowerment"),
            ("powerful", 0.70, 0.55, 0.85, "empowerment"),
            ("strong", 0.65, 0.40, 0.82, "empowerment"),
            ("strength", 0.68, 0.35, 0.80, "empowerment"),
            ("courage", 0.72, 0.50, 0.78, "empowerment"),
            ("courageous", 0.75, 0.55, 0.80, "empowerment"),
            ("brave", 0.72, 0.52, 0.78, "empowerment"),
            ("fearless", 0.78, 0.55, 0.82, "empowerment"),
            ("bold", 0.55, 0.60, 0.75, "empowerment"),
            ("confident", 0.65, 0.35, 0.80, "empowerment"),
            ("capable", 0.65, 0.20, 0.72, "empowerment"),
            ("competent", 0.60, 0.10, 0.70, "empowerment"),
            ("resilient", 0.68, 0.25, 0.75, "empowerment"),
            ("unstoppable", 0.78, 0.70, 0.85, "empowerment"),
            ("invincible", 0.75, 0.65, 0.85, "empowerment"),
            ("mighty", 0.72, 0.50, 0.80, "empowerment"),
            ("dominant", 0.30, 0.45, 0.85, "empowerment"),
            ("triumphant", 0.85, 0.78, 0.82, "empowerment"),
            ("victorious", 0.85, 0.75, 0.80, "empowerment"),
            // === VULNERABILITY ===
            ("vulnerable", -0.45, 0.30, -0.55, "vulnerability"),
            ("vulnerability", -0.42, 0.28, -0.52, "vulnerability"),
            ("fragile", -0.50, 0.10, -0.55, "vulnerability"),
            ("delicate", -0.20, 0.05, -0.30, "vulnerability"),
            ("exposed", -0.30, 0.55, -0.25, "vulnerability"),
            ("defenseless", -0.65, 0.30, -0.70, "vulnerability"),
            ("helpless", -0.70, 0.20, -0.75, "vulnerability"),
            ("powerless", -0.65, 0.10, -0.72, "vulnerability"),
            ("weak", -0.55, 0.05, -0.65, "vulnerability"),
            ("fragile", -0.50, 0.10, -0.55, "vulnerability"),
            ("sensitive", -0.15, 0.25, -0.20, "vulnerability"),
            ("tender", 0.60, 0.10, 0.15, "vulnerability"),
            ("raw", -0.10, 0.35, -0.15, "vulnerability"),
            ("bare", 0.00, 0.20, -0.10, "vulnerability"),
            ("naked", -0.10, 0.30, -0.20, "vulnerability"),
            ("wounded", -0.60, 0.20, -0.40, "vulnerability"),
            ("bruised", -0.50, 0.10, -0.35, "vulnerability"),
            // === NEUTRAL / LOW-VALENCE ===
            ("indifferent", -0.25, -0.45, -0.15, "neutral"),
            ("neutral", 0.00, 0.00, 0.00, "neutral"),
            ("objective", 0.10, -0.20, 0.30, "neutral"),
            ("impartial", 0.15, -0.25, 0.35, "neutral"),
            ("detached", -0.20, -0.40, 0.10, "neutral"),
            ("clinical", 0.05, -0.30, 0.25, "neutral"),
            ("analytical", 0.10, -0.10, 0.45, "neutral"),
            ("logical", 0.20, -0.20, 0.50, "neutral"),
            ("rational", 0.25, -0.15, 0.52, "neutral"),
            ("pragmatic", 0.30, -0.10, 0.55, "neutral"),
            ("stoic", 0.20, -0.35, 0.40, "neutral"),
            ("unemotional", 0.00, -0.50, 0.20, "neutral"),
            ("matter of fact", 0.10, -0.30, 0.30, "neutral"),
            ("factual", 0.10, -0.25, 0.35, "neutral"),
            ("ordinary", 0.20, -0.20, 0.10, "neutral"),
            ("routine", 0.15, -0.25, 0.15, "neutral"),
            ("normal", 0.30, -0.10, 0.30, "neutral"),
            ("typical", 0.15, -0.15, 0.20, "neutral"),
            ("standard", 0.10, -0.10, 0.25, "neutral"),
            ("usual", 0.15, -0.15, 0.20, "neutral"),
            ("expected", 0.10, -0.05, 0.15, "neutral"),
            ("predictable", -0.05, -0.30, 0.05, "neutral"),
            ("unsurprising", 0.00, -0.25, 0.10, "neutral"),
            ("commonplace", 0.05, -0.20, 0.05, "neutral"),
            // === DISBELIEF ===
            ("disbelief", -0.30, 0.70, -0.20, "surprise"),
            ("incredulous", -0.20, 0.72, -0.05, "surprise"),
            ("unconvinced", -0.20, 0.25, -0.10, "doubt"),
            ("nonplussed", -0.10, 0.30, -0.15, "surprise"),
            ("dubious", -0.25, 0.35, 0.05, "doubt"),
            ("questionable", -0.30, 0.30, -0.05, "doubt"),
            ("improbable", -0.20, 0.35, -0.10, "doubt"),
            ("implausible", -0.25, 0.40, -0.10, "doubt"),
            ("unlikely", -0.15, 0.20, -0.05, "doubt"),
            ("suspicious", -0.45, 0.50, -0.10, "doubt"),
            ("fishy", -0.40, 0.45, -0.05, "doubt"),
            ("shady", -0.50, 0.40, 0.00, "doubt"),
            // === APPROVAL ===
            ("approval", 0.68, 0.10, 0.50, "trust"),
            ("approve", 0.65, 0.12, 0.48, "trust"),
            ("praise", 0.75, 0.40, 0.55, "trust"),
            ("praiseworthy", 0.78, 0.35, 0.58, "trust"),
            ("commend", 0.72, 0.30, 0.55, "trust"),
            ("commendable", 0.75, 0.28, 0.56, "trust"),
            ("acclaim", 0.78, 0.50, 0.60, "trust"),
            ("accolade", 0.80, 0.45, 0.58, "trust"),
            ("endorse", 0.70, 0.20, 0.55, "trust"),
            ("endorsement", 0.68, 0.22, 0.52, "trust"),
            ("validation", 0.65, 0.15, 0.50, "trust"),
            ("validated", 0.68, 0.18, 0.52, "trust"),
            ("recognition", 0.72, 0.35, 0.60, "trust"),
            ("acknowledged", 0.60, 0.10, 0.45, "trust"),
            ("honored", 0.78, 0.30, 0.62, "trust"),
            // === DISAPPROVAL ===
            ("disapproval", -0.55, 0.35, 0.20, "disgust"),
            ("disapprove", -0.52, 0.32, 0.18, "disgust"),
            ("criticism", -0.50, 0.40, 0.15, "anger"),
            ("criticize", -0.52, 0.42, 0.18, "anger"),
            ("condemn", -0.68, 0.55, 0.30, "anger"),
            ("condemnation", -0.70, 0.58, 0.28, "anger"),
            ("denounce", -0.65, 0.60, 0.32, "anger"),
            ("denunciation", -0.68, 0.62, 0.30, "anger"),
            ("reprimand", -0.55, 0.45, 0.25, "anger"),
            ("rebuke", -0.58, 0.48, 0.28, "anger"),
            ("censure", -0.55, 0.40, 0.25, "anger"),
            ("castigate", -0.60, 0.55, 0.30, "anger"),
            ("chastise", -0.55, 0.50, 0.28, "anger"),
            ("objection", -0.45, 0.40, 0.20, "disgust"),
            ("protest", -0.35, 0.60, 0.35, "anger"),
            // === SURPRISE: POSITIVE ===
            ("delighted", 0.90, 0.55, 0.65, "joy"),
            ("pleasantly surprised", 0.70, 0.75, 0.45, "surprise"),
            ("overjoyed", 0.95, 0.70, 0.72, "joy"),
            ("thrilled", 0.90, 0.80, 0.70, "joy"),
            ("ecstatic", 0.95, 0.80, 0.78, "joy"),
            ("elated", 0.92, 0.70, 0.75, "joy"),
            ("exhilarated", 0.90, 0.85, 0.72, "joy"),
            ("excited", 0.80, 0.85, 0.60, "joy"),
            ("animated", 0.65, 0.65, 0.55, "joy"),
            ("vibrant", 0.72, 0.60, 0.60, "joy"),
            ("spirited", 0.70, 0.65, 0.62, "joy"),
            ("enthusiastic", 0.78, 0.75, 0.65, "joy"),
            ("zealous", 0.55, 0.78, 0.68, "joy"),
            ("ardent", 0.60, 0.65, 0.60, "joy"),
            ("fervent", 0.50, 0.72, 0.62, "joy"),
            // === SYMPATHY (continued) ===
            ("condolence", 0.20, -0.05, -0.10, "sympathy"),
            ("condolences", 0.15, -0.08, -0.12, "sympathy"),
            ("commiseration", 0.30, 0.05, 0.00, "sympathy"),
            ("commiserate", 0.25, 0.08, 0.02, "sympathy"),
            ("fellow feeling", 0.50, 0.10, 0.20, "sympathy"),
            ("understanding", 0.60, 0.10, 0.55, "sympathy"),
            ("supportive", 0.70, 0.10, 0.45, "sympathy"),
            ("reassuring", 0.68, -0.15, 0.48, "sympathy"),
            ("comforting", 0.72, -0.20, 0.40, "sympathy"),
            ("consoling", 0.65, -0.10, 0.35, "sympathy"),
            ("soothing", 0.68, -0.55, 0.38, "sympathy"),
            // === POWER DYNAMICS ===
            ("power", 0.50, 0.45, 0.85, "power"),
            ("powerless", -0.65, 0.10, -0.72, "power"),
            ("control", 0.40, 0.20, 0.80, "power"),
            ("controlled", 0.10, 0.15, 0.55, "power"),
            ("controlling", -0.30, 0.40, 0.70, "power"),
            ("dominate", 0.10, 0.55, 0.78, "power"),
            ("domination", -0.10, 0.50, 0.75, "power"),
            ("submissive", -0.35, -0.20, -0.55, "power"),
            ("submission", -0.40, -0.10, -0.50, "power"),
            ("obedient", 0.10, -0.20, -0.30, "power"),
            ("defiant", 0.10, 0.65, 0.55, "power"),
            ("rebellious", 0.05, 0.70, 0.50, "power"),
            ("rebel", 0.20, 0.68, 0.55, "power"),
            ("authority", 0.35, 0.25, 0.78, "power"),
            ("authoritative", 0.30, 0.30, 0.80, "power"),
            ("dominant", 0.30, 0.45, 0.85, "power"),
            ("mighty", 0.72, 0.50, 0.80, "power"),
            ("weak", -0.55, 0.05, -0.65, "power"),
            ("subordinate", -0.20, -0.10, -0.40, "power"),
            ("inferior", -0.45, 0.10, -0.50, "power"),
            ("superior", -0.20, 0.15, 0.75, "power"),
            ("leadership", 0.60, 0.35, 0.78, "power"),
            ("leader", 0.60, 0.40, 0.80, "power"),
            ("follower", 0.20, -0.10, -0.20, "power"),
            // === CONFUSION / UNCERTAINTY MISC ===
            ("confusing", -0.35, 0.45, -0.25, "confusion"),
            ("unclear", -0.20, 0.20, -0.25, "confusion"),
            ("ambiguous", -0.10, 0.15, -0.15, "confusion"),
            ("vague", -0.15, 0.10, -0.20, "confusion"),
            ("murky", -0.35, 0.20, -0.20, "confusion"),
            ("hazy", -0.20, 0.10, -0.20, "confusion"),
            ("foggy", -0.15, 0.05, -0.15, "confusion"),
            ("obscure", -0.25, 0.15, -0.15, "confusion"),
            ("cryptic", -0.15, 0.35, 0.05, "confusion"),
            ("enigmatic", -0.05, 0.30, 0.10, "confusion"),
            ("mysterious", 0.10, 0.45, 0.05, "confusion"),
            ("mystery", 0.20, 0.50, 0.10, "confusion"),
            ("paradox", 0.10, 0.45, 0.15, "confusion"),
            ("contradiction", -0.20, 0.50, 0.00, "confusion"),
            ("inconsistent", -0.30, 0.35, -0.10, "confusion"),
            ("incoherent", -0.45, 0.30, -0.25, "confusion"),
            ("nonsensical", -0.40, 0.35, -0.10, "confusion"),
            ("absurd", -0.20, 0.60, 0.15, "confusion"),
            ("illogical", -0.35, 0.30, -0.05, "confusion"),
            ("irrational", -0.40, 0.45, 0.00, "confusion"),
            // === URGENCY ===
            ("urgent", 0.10, 0.80, 0.45, "urgency"),
            ("urgency", 0.05, 0.78, 0.42, "urgency"),
            ("critical", -0.10, 0.75, 0.50, "urgency"),
            ("crucial", 0.15, 0.70, 0.55, "urgency"),
            ("vital", 0.35, 0.65, 0.60, "urgency"),
            ("imperative", 0.20, 0.68, 0.58, "urgency"),
            ("pressing", 0.00, 0.72, 0.40, "urgency"),
            ("imminent", -0.15, 0.75, 0.20, "urgency"),
            ("impending", -0.20, 0.72, 0.10, "urgency"),
            ("looming", -0.25, 0.70, 0.05, "urgency"),
            ("emergent", 0.15, 0.65, 0.35, "urgency"),
            ("emergency", -0.30, 0.85, 0.20, "urgency"),
            ("crisis", -0.50, 0.82, 0.10, "urgency"),
            ("desperate", -0.70, 0.78, -0.20, "urgency"),
            ("last minute", -0.10, 0.75, 0.15, "urgency"),
            ("race against", 0.10, 0.80, 0.40, "urgency"),
            ("now or never", 0.05, 0.82, 0.35, "urgency"),
            ("hurry", -0.05, 0.72, 0.30, "urgency"),
            ("rush", -0.05, 0.70, 0.28, "urgency"),
            // === CONFIDENCE CONTINUED ===
            ("sure", 0.50, 0.15, 0.70, "certainty"),
            ("certain", 0.55, 0.20, 0.75, "certainty"),
            ("positive", 0.60, 0.30, 0.68, "certainty"),
            ("definite", 0.50, 0.20, 0.72, "certainty"),
            ("absolute", 0.30, 0.40, 0.78, "certainty"),
            ("unquestionable", 0.50, 0.25, 0.80, "certainty"),
            ("indisputable", 0.45, 0.28, 0.78, "certainty"),
            ("irrefutable", 0.45, 0.30, 0.80, "certainty"),
            ("undeniable", 0.40, 0.35, 0.72, "certainty"),
            ("guaranteed", 0.45, 0.20, 0.75, "certainty"),
            ("inevitable", -0.10, 0.40, 0.50, "certainty"),
            ("assured", 0.55, 0.10, 0.70, "certainty"),
        ]
    }

    pub fn analyze(&self, text: &str) -> Vec<(&'static str, (f64, f64, f64, &'static str))> {
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        let mut hits = Vec::new();
        for word in &words {
            let clean = word.trim_matches(|c: char| !c.is_alphabetic());
            if let Some((key, entry)) = self.valence_map.get_key_value(clean) {
                hits.push((*key, *entry));
            }
        }
        hits
    }

    fn is_negation(&self, word: &str) -> bool {
        self.negation_markers.contains(&word)
    }

    fn get_intensity(&self, word: &str) -> Option<f64> {
        self.intensity_modifiers.get(word).copied()
    }
}

#[derive(Debug, Clone)]
pub struct EmotionAnalyzer {
    lexicon: EmotionLexicon,
}

impl Default for EmotionAnalyzer {
    fn default() -> Self {
        Self {
            lexicon: EmotionLexicon::default(),
        }
    }
}

impl EmotionAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tag_text(&self, text: &str, source: &str) -> EmotionTag {
        let lower = text.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();
        let words: Vec<&str> = tokens
            .iter()
            .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
            .collect();

        let mut hit_indices = Vec::new();
        for (i, word) in words.iter().enumerate() {
            if word.is_empty() {
                continue;
            }
            if self.lexicon.valence_map.contains_key(word) {
                if self.lexicon.is_negation(word) || self.lexicon.get_intensity(word).is_some() {
                    continue;
                }
                hit_indices.push(i);
            }
        }

        if hit_indices.is_empty() {
            return EmotionTag {
                valence: 0.0,
                arousal: 0.0,
                dominance: 0.0,
                primary_emotion: "neutral".into(),
                trigger_source: source.to_string(),
                word_hits: Vec::new(),
            };
        }

        let mut sum_v = 0.0;
        let mut sum_a = 0.0;
        let mut sum_d = 0.0;
        let mut emotion_counts: HashMap<&str, usize> = HashMap::new();
        let mut word_hits = Vec::new();

        for &i in &hit_indices {
            let word = words[i];
            let (v, a, d, emo) = self.lexicon.valence_map[word];

            let mut final_v = v;
            let mut final_a = a;
            let mut final_d = d;

            let search_start = if i >= 3 { i - 3 } else { 0 };

            let negated = words[search_start..i]
                .iter()
                .any(|w| self.lexicon.is_negation(w));
            if negated {
                final_v = -final_v;
                final_a *= 0.5;
            }

            let mut intensity_mult: f64 = 1.0;
            for j in (search_start..i).rev() {
                if let Some(mult) = self.lexicon.get_intensity(words[j]) {
                    intensity_mult = mult;
                    break;
                }
            }
            if intensity_mult != 1.0 {
                final_v = (final_v * intensity_mult).clamp(-1.0, 1.0);
                final_a = (final_a * intensity_mult).clamp(-1.0, 1.0);
                final_d = (final_d * intensity_mult).clamp(-1.0, 1.0);
            }

            sum_v += final_v;
            sum_a += final_a;
            sum_d += final_d;
            *emotion_counts.entry(emo).or_insert(0) += 1;
            word_hits.push(word.to_string());
        }

        let n = hit_indices.len() as f64;
        let primary = emotion_counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(e, _)| e.to_string())
            .unwrap_or_else(|| "neutral".into());

        EmotionTag {
            valence: (sum_v / n).clamp(-1.0, 1.0),
            arousal: (sum_a / n).clamp(-1.0, 1.0),
            dominance: (sum_d / n).clamp(-1.0, 1.0),
            primary_emotion: primary,
            trigger_source: source.to_string(),
            word_hits,
        }
    }

    pub fn valence_label(valence: f64) -> &'static str {
        if valence > 0.5 {
            "positive"
        } else if valence < -0.5 {
            "negative"
        } else {
            "neutral"
        }
    }

    pub fn arousal_label(arousal: f64) -> &'static str {
        if arousal > 0.5 {
            "high"
        } else if arousal < -0.3 {
            "low"
        } else {
            "moderate"
        }
    }

    pub fn is_emotionally_loaded(&self, text: &str, threshold: f64) -> bool {
        let tag = self.tag_text(text, "check");
        tag.valence.abs() > threshold || tag.arousal > threshold
    }

    pub fn summarize(&self, tag: &EmotionTag) -> String {
        format!(
            "[emotion] {} | valence={:.2} ({}) | arousal={:.2} ({}) | hits={}",
            tag.primary_emotion,
            tag.valence,
            Self::valence_label(tag.valence),
            tag.arousal,
            Self::arousal_label(tag.arousal),
            tag.word_hits.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_text() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("I feel happy and delighted with this wonderful joy", "test");
        assert_eq!(tag.primary_emotion, "joy");
        assert!(tag.valence > 0.5);
    }

    #[test]
    fn test_negative_text() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text(
            "This is a disaster, a terrible catastrophe of corruption and betrayal",
            "test",
        );
        assert!(tag.valence < -0.5);
        assert!(tag.arousal > 0.3);
    }

    #[test]
    fn test_neutral_text() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("The meeting is scheduled for Tuesday at 3pm", "test");
        assert_eq!(tag.primary_emotion, "neutral");
    }

    #[test]
    fn test_emotionally_loaded_detection() {
        let a = EmotionAnalyzer::new();
        assert!(a.is_emotionally_loaded("This is an outrage and a catastrophe", 0.5));
        assert!(!a.is_emotionally_loaded("Please pass the salt", 0.5));
    }

    #[test]
    fn test_summarize() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("I am so happy and grateful", "source_a");
        let s = a.summarize(&tag);
        assert!(s.contains("emotion"));
        assert!(s.contains("valence"));
    }

    #[test]
    fn test_negation_flips_valence() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("I am not happy", "test");
        assert!(
            tag.valence < 0.0,
            "negation should flip valence to negative, got {}",
            tag.valence
        );
        assert_eq!(tag.primary_emotion, "joy");
    }

    #[test]
    fn test_negation_within_3_words() {
        let a = EmotionAnalyzer::new();
        let _tag = a.tag_text("This is not a good situation and it is terrible", "test");
        let positive = a.tag_text("a good situation", "test");
        let negative = a.tag_text("not a good situation", "test");
        assert!(
            negative.valence < positive.valence,
            "negation should reduce valence"
        );
    }

    #[test]
    fn test_negation_no_effect_without_marker() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("I am deeply happy about this", "test");
        assert!(tag.valence > 0.5);
    }

    #[test]
    fn test_intensity_modifier_amplifies() {
        let a = EmotionAnalyzer::new();
        let neutral = a.tag_text("I am happy", "test");
        let intense = a.tag_text("I am very happy", "test");
        assert!(
            intense.valence.abs() > neutral.valence.abs(),
            "intensity modifier should amplify valence: neutral={}, intense={}",
            neutral.valence,
            intense.valence
        );
    }

    #[test]
    fn test_extreme_intensity_modifier() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("This is extremely terrible", "test");
        assert!(
            tag.valence < -0.7,
            "extreme + terrible should be very negative"
        );
    }

    #[test]
    fn test_somewhat_reduces_intensity() {
        let a = EmotionAnalyzer::new();
        let plain = a.tag_text("I am sad", "test");
        let reduced = a.tag_text("I am somewhat sad", "test");
        assert!(
            reduced.valence.abs() < plain.valence.abs(),
            "somewhat should reduce intensity: plain={}, reduced={}",
            plain.valence,
            reduced.valence
        );
    }

    #[test]
    fn test_slightly_reduces_intensity() {
        let a = EmotionAnalyzer::new();
        let plain = a.tag_text("I am angry", "test");
        let reduced = a.tag_text("I am slightly angry", "test");
        assert!(
            reduced.valence.abs() < plain.valence.abs(),
            "slightly should reduce intensity"
        );
    }

    #[test]
    fn test_negation_with_intensity() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("I am not very happy about this", "test");
        assert!(
            tag.valence < 0.0,
            "not very happy should be negative, got {}",
            tag.valence
        );
    }

    #[test]
    fn test_negation_arousal_halved() {
        let a = EmotionAnalyzer::new();
        let plain = a.tag_text("I am terrified", "test");
        let negated = a.tag_text("I am not terrified", "test");
        assert!(
            negated.arousal < plain.arousal,
            "negation should reduce arousal: plain={}, negated={}",
            plain.arousal,
            negated.arousal
        );
    }

    #[test]
    fn test_emotion_word_count() {
        let count = EmotionLexicon::lexicon_data().len();
        assert!(
            count >= 500,
            "lexicon should have 500+ words, got {}",
            count
        );
    }

    #[test]
    fn test_negation_markers_excluded_from_hits() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("not no never nothing", "test");
        assert_eq!(tag.primary_emotion, "neutral");
        assert!(tag.word_hits.is_empty());
    }

    #[test]
    fn test_intensity_modifier_clamps() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("This is extremely wonderful amazing fantastic joy", "test");
        assert!(
            tag.valence <= 1.0,
            "valence should be clamped to 1.0, got {}",
            tag.valence
        );
        assert!(tag.valence > 0.5);
    }

    #[test]
    fn test_deeply_modifier() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("I am deeply sad about this loss", "test");
        assert!(tag.valence < -0.5);
    }

    #[test]
    fn test_original_tests_still_pass() {
        test_positive_text();
        test_negative_text();
        test_neutral_text();
        test_emotionally_loaded_detection();
        test_summarize();
    }

    #[test]
    fn test_nothing_negation() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("nothing good came of this", "test");
        assert!(
            tag.valence <= 0.0 || tag.primary_emotion == "neutral",
            "nothing should negate good: valence={}",
            tag.valence
        );
    }

    #[test]
    fn test_disinformation_signals() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text(
            "This is a fraudulent conspiracy with corrupt cover-up",
            "test",
        );
        assert!(
            tag.valence < -0.3,
            "disinformation words should be negative"
        );
        assert!(!tag.word_hits.is_empty());
    }

    #[test]
    fn test_moral_emotions() {
        let a = EmotionAnalyzer::new();
        let tag = a.tag_text("I feel righteous indignation about this injustice", "test");
        assert!(tag.primary_emotion == "anger" || tag.valence.abs() > 0.3);
    }
}
