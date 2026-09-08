#!/usr/bin/env python3
"""
Batch URL Import to NeoTrix KB
分类并导入URL到知识库
"""

import subprocess
import json
import re
from typing import List, Dict, Tuple
from dataclasses import dataclass
from enum import Enum

class URLCategory(Enum):
    GITHUB_REPO = "github_repo"
    ARXIV_PAPER = "arxiv_paper"
    HUGGINGFACE = "huggingface"
    TOOL_FRAMEWORK = "tool_framework"
    BLOG_ARTICLE = "blog_article"
    DOCUMENTATION = "documentation"
    UNKNOWN = "unknown"

@dataclass
class URLInfo:
    url: str
    category: URLCategory
    title: str
    description: str
    stars: int
    tags: List[str]

# URL分类关键词
CATEGORY_KEYWORDS = {
    URLCategory.GITHUB_REPO: ["github.com"],
    URLCategory.ARXIV_PAPER: ["arxiv.org", "biorxiv.org"],
    URLCategory.HUGGINGFACE: ["huggingface.co"],
    URLCategory.TOOL_FRAMEWORK: ["readthedocs.io", "docs.", "devdocs.io"],
    URLCategory.BLOG_ARTICLE: ["blog.", "medium.com", "substack.com"],
    URLCategory.DOCUMENTATION: ["docs.", "documentation"],
}

# 提取的URL列表 (从用户输入)
URLS_RAW = """
https://www.biorxiv.org/content/10.64898/2026.01.09.698608v1
https://huggingface.co/papers/2609.01437
https://github.com/Michael-A-Kuykendall/shimmy
https://www.nature.com/articles/s41586-026-10898-6
https://github.com/zartbot/blog/issues/14
https://vrg.fel.cvut.cz/reimagenet
https://arxiv.org/abs/2608.13783
https://github.com/Thysrael/Horizon
https://github.com/zhaoxuya520/reverse-skill
https://itsfree.ai/
https://github.com/bikini/exploitarium
https://github.com/2025Emma/vibe-coding-cn
https://www.liquid.ai/blog/introducing-liquid-nanos-frontier-grade-performance-on-everyday-devices
https://arxiv.org/abs/2608.26537
https://francesco215.github.io/Scacchi/
https://arxiv.org/abs/2608.30320
https://github.com/tonhowtf/omniget
https://github.com/Ephemeral-AI-Lab/layerfs
https://www.biorxiv.org/content/10.64898/2026.09.01.748703v1
https://www.biorxiv.org/alertsrss
https://github.com/stardustai/dataset-viewer
https://github.com/deeplethe/utopia
https://eprint.iacr.org/2026/1849
https://www.boozallen.com/insights/cyber/cyber-weapon-index.html
https://github.com/langchain-ai/agents-from-scratch
https://hackers-arise.com/linux-hackshell-bash-for-hackers/
https://github.com/zakirkun/deep-eye
https://github.com/OpenAEC-Foundation/open-pdf-studio/
https://x.ai/news/designing-grok-bot
https://traces.apodex.com/
https://github.com/magnitudedev/magnitude
https://github.com/GiMi-Xiaomi/gimi-illustration-skill
https://github.com/genspark-ai/genoffice
https://github.com/0x6rss/instagram-private-graph
https://arxiv.org/abs/2609.00865
https://arxiv.org/abs/2609.02737
https://github.com/vava-nessa/free-coding-models
https://docs.priorlabs.ai/cookbook/decoder_readout?utm_source=socials
https://github.com/eternityspring/shuohao-skills
https://github.com/usestrix/strix
https://www.palantir.com/docs/foundry/architecture-center/overview
https://lab.noematrix.ai/blog/1-noe-0-research-preview/
https://self-developing-agents.github.io/
https://github.com/echonoshy/cgft-llm
https://huggingface.co/papers/2609.01437
https://github.com/Stremio/stremio-web
https://paperswithcode.co/paper/2609.00638
https://arxiv.org/pdf/2504.17033
https://openai.com/index/gpt-6-astra/
https://www.nature.com/articles/s42256-026-01286-w
https://github.com/lordx64/pentestkit
https://github.com/mrbuzzoni/loop-rat
https://github.com/orbi-build/orbi
https://github.com/synthetic-sciences/openscience
https://huggingface.co/datasets/echel0nn1881/kimi-cyber-reasoning
https://github.com/2akouwu/reverify
https://github.com/Chocobozzz/PeerTube
https://arxiv.org/abs/2609.02042
https://github.com/topics/dsh-plugin
https://github.com/cobusgreyling/loop-engineering
https://github.com/NirDiamant/Agent_Memory_Techniques
https://github.com/Sumanth077/Hands-On-AI-Engineering
https://x.com/HuggingPapers/status/2095425335915598198
https://huggingface.co/junchaoh-cs/SolarWM
https://paperswithcode.co/paper/2609.02886
https://huggingface.co/datasets/junchaoh-cs/SolarWM-Data
https://github.com/DannyMac180/fable-advisor
https://arxiv.org/pdf/2608.30509
https://github.com/acamposuribe/p5.brush
https://github.com/Nutlope/open-customer-insights
https://github.com/oso95/scroll-world
https://github.com/ReversecLabs/virtual.attack
https://github.com/ryanmcdermott/clean-code-javascript
https://arxiv.org/abs/2608.25593
https://github.com/3b1b/manim
https://github.com/robert-mcdermott/ai-knowledge-graph
https://huggingface.co/papers/2609.02737
https://github.com/sentrux/sentrux
https://github.com/inkeep/open-knowledge
https://github.com/s0ld13rr/pentestcode
https://github.com/Open-LLM-VTuber/Open-LLM-VTuber
https://github.com/openai/codex/pull/27488#event-26606217582
https://github.com/microsoft/RD-Agent
https://github.com/IvanMurzak/Unity-MCP
https://mthli.xyz/git-knowledge-loop/
https://best.xiaohu.ai/article/xai-bot-guides/
https://github.com/buka-studio/www-marijanapav
https://github.com/ahujasid/blender-mcp
https://github.com/apurvsinghgautam/robin
https://github.com/superlinked/sie
https://github.com/PurpleAILAB/Decepticon
https://github.com/Armur-Ai/Pentest-Swarm-AI
https://github.com/usestrix/strix
https://arxiv.org/abs/2608.28293
https://arxiv.org/abs/2608.28476
https://arxiv.org/abs/2608.28233
https://github.com/zuruoke/watermark-removal
https://github.com/h4ckf0r0day/obscura
https://github.com/WooooDyy/LLM-Agent-Paper-List
https://arxiv.org/abs/2608.28444
https://github.com/rawfilejson/awesome-osint-arsenal
https://www.alphaxiv.org/abs/2608.31036
https://arxiv.org/abs/2608.30384
https://github.com/sidinsearch/superbrain
https://arxiv.org/abs/2608.19197
https://arxiv.org/abs/2608.30163
https://github.com/zarazhangrui/follow-builders/tree/main
https://arxiv.org/abs/2608.13940
https://github.com/hughhowey/neo
https://arxiv.org/pdf/2411.01537
https://github.com/coda0HQ/open-artifacts
https://github.com/echoVic/orca-agent
https://arxiv.org/abs/2608.28444
https://github.com/webtorrent/webtorrent
https://github.com/firecrawl/anydoc
https://github.com/browseros-ai/BrowserOS
https://github.com/Human-Agent-Society/reef/tree/main/reef
https://github.com/echohive42/AI-reads-books-page-by-page
https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5-1
https://github.com/Michael-A-Kuykendall/shimmy
https://github.com/jingyaogong/minimind
https://github.com/rahulnyk/knowledge_graph
https://github.com/Sumanth077/Hands-On-AI-Engineering/tree/main/ai_agents/deep_research_assistant
https://arxiv.org/abs/2608.26991
https://arxiv.org/abs/2608.29530
https://github.com/deeplethe/utopia
https://github.com/CyberStrikeus/CyberStrike
https://github.com/covan-ai/covan
https://github.com/zvec-ai/zvec-grep
https://github.com/tw93/Pake
https://github.com/sooryathejas/METATRON
https://github.com/Gnosil/semantix
https://github.com/pranshuparmar/witr
https://github.com/jingyaogong/minimind
https://github.com/sapientinc/praxist
https://github.com/firecrawl/pdf-inspector
https://github.com/DietrichGebert/ponytail
https://github.com/THU-MAIC/OpenMAIC
https://github.com/crmne/fastpotify
https://github.com/4242labs/tokentab
https://github.com/jprx/darwin-vm
https://github.com/vitali87/code-graph-rag
https://arxiv.org/abs/2609.01325
https://github.com/yizhiyanhua-ai/fireworks-tech-graph
https://github.com/codingagentsystem/cas
https://arxiv.org/abs/2609.00198
https://arxiv.org/abs/2608.30310
https://github.com/AzeemIdrisi/PhoneSploit-Pro
https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-fable-5-1?utm_source=chatgpt.com
https://github.com/gepa-ai/gepa
https://arxiv.org/pdf/2606.19857
https://github.com/LanRhyme/MicYou
https://github.com/Imbad0202/academic-research-skills
https://github.com/elementalsouls/Claude-BugHunter
https://github.com/kydlikebtc/awesome-grokbot
https://github.com/h4ckf0r0day/obscura?ref=selfh.st
https://github.com/MaxMiksa/Auto-Company
https://github.com/khoj-ai/khoj
https://arxiv.org/pdf/2410.18417
https://github.com/EthanYoQ/Invoice-Downloader
https://github.com/anthropics/commerce-agents
https://arxiv.org/abs/2608.23670
arxiv.org/abs/2608.26747
https://github.com/Oihalitz/xdp-dns-evadeproxy
https://github.com/ifixai-ai/iFixAi
https://arxiv.org/abs/2608.27266
https://github.com/Netw0rkNoob/VulnClaw
https://arxiv.org/abs/2606.11680
https://github.com/Vincentwei1021/video-shotcraft
https://arxiv.org/abs/2608.24735
https://github.com/EasyTier/EasyTier
https://jelectrochem.xmu.edu.cn/journal/vol32/iss6/1/
https://github.com/talesofai/cohub
https://github.com/lobu-ai/lobu
https://arxiv.org/abs/1703.06114
arxiv.org/abs/2608.25593
https://www.alphaxiv.org/shared/folder/01a05f55-d998-7e4f-a6be-0990e8c040e1
https://github.com/hardbeat920/monocode
https://github.com/zvec-ai/zvec-grep
https://github.com/penligent/AI2PentestTool
https://www.alphaxiv.org/abs/2608.31036
https://github.com/opendataloader-project/opendataloader-pdf
https://github.com/Firma-AI/openfirma
https://github.com/vibrantlabsai/ragas
https://github.com/567-labs/instructor
https://github.com/Unstructured-IO/unstructured
https://github.com/Unstructured-IO/unstructured
https://github.com/567-labs/instructor
https://github.com/Unstructured-IO/unstructured
https://github.com/kirodotdev/KiroCrew
https://github.com/sodiumsun/agenttrail
arxiv.org/abs/2608.26204
https://paperswithcode.co/paper/2609.01343
https://github.com/anthropics/claude-code/tree/main
https://arxiv.org/abs/2608.28128
https://github.com/Justin-sky/ai-art-engine
https://github.com/lipku/LiveTalking
https://github.com/addyosmani/agent-skills
https://github.com/nextlevelbuilder/ui-ux-pro-max-skill
https://github.com/JuliusBrussee/caveman
https://github.com/kepano/obsidian-skills
https://github.com/alchaincyf/nuwa-skill
https://github.com/martin-olivier/airgorah
https://github.com/apurvsinghgautam/robin
https://github.com/touchine-ojo/OJO-Design-Skills
https://www.dwarkesh.com/p/openai-huggingface
https://github.com/DietrichGebert/ponytail
https://arxiv.org/abs/2608.27964
https://vercel.com/blog/how-our-agents-build-on-brand-pages-with-design-md
https://github.com/wolfpld/tracy
https://arxiv.org/abs/2608.27991
https://github.com/lijigang/ljg-skills
https://academy.dair.ai/papers/wikiskill-compiles-agent-experience-into-a-persistent-wiki-2608.27454
https://github.com/cbrock84/headcount
https://arxiv.org/abs/2608.18027
https://arxiv.org/html/2606.05608v1
https://paperswithcode.co/paper/2608.28122
https://github.com/bigskysoftware/htmx
https://github.com/OpenHands/OpenHands
https://github.com/elementalsouls/Claude-OSINT
https://github.com/opopile/beichen-pi-desktop
https://github.com/DietrichGebert/ponytail
https://github.com/tt-a1i/archify
https://github.com/max-sixty/worktrunk
https://github.com/THU-MAIC/OpenMAIC
https://github.com/FlashML-org/FreeToken
https://github.com/elder-plinius/G0DM0D3
https://github.com/www222fff/free-router
https://github.com/pkuflyingpig/cs-self-learning/
https://github.com/tirth8205/code-review-graph
https://github.com/Mibayy/token-savior
https://github.com/mksglu/context-mode
https://github.com/rtk-ai/rtk
https://github.com/ooples/token-optimizer-mcp
https://github.com/zilliztech/claude-context
https://github.com/nadimtuhin/claude-token-optimizer
https://github.com/alexgreensh/token-optimizer
https://github.com/drona23/claude-token-efficient
https://github.com/JuliusBrussee/caveman
https://github.com/google/adk-python
https://github.com/alibaba/zvec
https://arxiv.org/abs/2608.21265
https://arxiv.org/abs/2606.11660
https://github.com/EleutherAI/bergson
https://bergson.readthedocs.io/en/latest/
arxiv.org/abs/2608.19920
https://arxiv.org/pdf/2510.07880
https://github.com/affaan-m/ECC
https://github.com/Jesseovo/last30days-skill-cn
https://github.com/lss233/kirara-ai
https://github.com/xai-org/grok-prompts
https://github.com/b-nnett/grok-bot
https://github.com/aaron-he-zhu/aaron-marketing-skills
https://github.com/milind-soni/OpenMausBot
https://github.com/elie222/rakazo
https://github.com/Renset/macai
https://github.com/workweave/router
https://github.com/Mininglamp-AI/Mano-P
https://github.com/DeusData/codebase-memory-mcp
https://github.com/karpathy/autoresearch
https://github.com/mattpocock/skills
https://github.com/Ephemeral-AI-Lab/layerfs
https://arxiv.org/abs/2608.26263
https://arxiv.org/abs/2608.26263
https://github.com/jingyaogong/minimind
https://arxiv.org/abs/2608.29899
https://arxiv.org/abs/2608.30949
https://github.com/omnigent-ai/omnigent
https://github.com/KnockOutEZ/wigolo
https://github.com/chen-985211/cleancode
https://github.com/ansible/ansible
arxiv.org/abs/2608.26530
https://github.com/Egonex-AI/Understand-Anything
https://github.com/getsentry/sentry
https://github.com/apurvsinghgautam/robin
https://github.com/antvis/Infographic
https://github.com/S1N6H/pentest-harness
https://github.com/sigpanic/goink
https://arxiv.org/abs/2608.23670
https://github.com/zhaoxuya520/reverse-skill
https://arxiv.org/pdf/2602.24286
https://github.com/DavidCarliez/trustmebro
https://github.com/huhusmang/Awesome-LLMs-for-Vulnerability-Detection
https://github.com/NirDiamant/GenAI_Agents
https://github.com/omkarcloud/google-maps-scraper
https://github.com/topics/google-maps
https://github.com/fadewalk/serenity-stock-choke
https://huggingface.co/papers/2608.27549
https://github.com/Hmbown/CodeWhale
https://arxiv.org/abs/2608.27454
https://github.com/theswerd/brainless
https://github.com/heygen-com/hyperframes
https://github.com/pollen-robotics/microduck
arXiv:2608.27351
https://blog.bytebytego.com/p/how-to-make-llms-3x-faster
https://simonwillison.net/2026/Feb/23/agentic-engineering-patterns/
https://simonwillison.net/guides/agentic-engineering-patterns/
https://github.com/bcefghj/multi-agent-aiops
https://github.com/MadsLorentzen/ai-job-search
https://arxiv.org/abs/2608.23265
https://github.com/fujiapple852/trippy
https://github.com/abi/screenshot-to-code
https://github.com/Adam-CAD/CADAM
https://github.com/markdown-viewer/skills
https://github.com/odysseus-dev/odysseus
https://github.com/usememos/memos
https://www.uber.com/us/en/blog/efficient-software-factory/
https://github.com/Whispergate/InfraGuard
https://github.com/Osmantic/ODS
https://github.com/garychowcmu/daizhigev20
https://www.alphaxiv.org/abs/2608.27448
https://www-cdn.anthropic.com/7b1c44894e980876479947dcdd40716278aeeffd/automated-alignment-researchers-august-2026.pdf
https://arxiv.org/pdf/2608.26701
https://github.com/qusong0627/QuantMind
https://github.com/any4ai/anycrawl
https://github.com/daytonaio/daytona
https://github.com/warpdotdev/common-skills
https://github.com/MakazhanAlpamys/Soup
https://github.com/smicallef/spiderfoot
https://arxiv.org/abs/2608.25593
https://github.com/martin-olivier/airgorah
https://github.com/yding-git/personal-edge-proxy
https://github.com/apurvsinghgautam/robin
https://github.com/bingreeky/JIT
https://arxiv.org/abs/2303.11366
https://github.com/noahshinn/reflexion
https://arxiv.org/abs/2304.03442
https://github.com/joonspk-research/generative_agents
https://arxiv.org/abs/2310.08560
https://github.com/letta-ai/letta
https://arxiv.org/abs/2504.19413
https://github.com/mem0ai/mem0
https://github.com/tfatykhov/awesome-agent-memory
https://github.com/Haozhe-Xing/agent_learning
https://github.com/pollen-robotics/microduck_rl
https://github.com/LodyAI/Lody
https://github.com/conorbronsdon/avoid-ai-writing
https://github.com/Awarexone/Agentic-Bug-Hunter
https://skillhub.cn/hermes/skill/tech-bug-troubleshooting
https://github.com/putyy/res-downloader
https://github.com/S1N6H/pentest-harness
https://github.com/abi/screenshot-to-code
https://github.com/aAAaqwq/AGI-Super-Team
https://arxiv.org/abs/2608.25776
https://genai-security-project.github.io/crosswalk/
https://github.com/Stirling-Tools/Stirling-PDF
https://arxiv.org/abs/2608.27391
https://github.com/iAmCorey/wake
https://reilabs.org/blog/emergence-toward-autonomous-structure-discovery
https://github.com/elizaos/eliza
https://github.com/The-PR-Agent/pr-agent
https://github.com/plandex-ai/plandex
https://www.alphaxiv.org/abs/2608.25927
https://www.alphaxiv.org/abs/2608.26105
https://arxiv.org/abs/2608.25593
https://claude.com/blog/how-warp-builds-self-improving-agents-on-claude
https://github.com/LinklyAI/best-skills
https://arxiv.org/abs/2608.18184
https://www.alphaxiv.org/abs/2608.26058
https://github.com/leonxlnx/taste-skill
https://arxiv.org/pdf/2606.26294
https://huggingface.co/papers/2608.11341
https://github.com/agentskills/agentskills
https://paperswithcode.co/paper/2608.26005
https://github.com/FareedKhan-dev/train-llm-from-scratch
https://arxiv.org/pdf/2604.10027
https://github.com/AgriciDaniel/claude-seo
https://arxiv.org/abs/2601.01885
https://github.com/MiroMindAI
https://monid.ai/SKILL.md
https://qingkeai.online/blog/JIT-Agent?utm_source=twitter
https://qingkeai.online/categories/jing-xuan-wen-zhang
https://github.com/remotion-dev/remotion
https://github.com/ScriptedAlchemy/grok-bot-cli
https://github.com/jeffhuber/grokbot-imessage-skill
https://github.com/b-nnett/grok-bot-0.18-reconstructed
https://github.com/RongleCat/awesome-grok-bot/blob/main/README.zh.md
https://github.com/echohive42/AI-reads-books-page-by-page
https://arxiv.org/abs/2608.25776
https://github.com/Hamed233/Cybersecurity-Mastery-Roadmap
https://github.com/ultrasecurity/Storm-Breaker
https://paperswithcode.co/paper/2608.15763
https://research.google/blog/planetary-prediction-engine-automating-global-models-via-earth-ai/?utm_source=twitter&utm_medium=social&utm_campaign=social_post&utm_content=gr-acct
https://github.com/xai-org/x-algorithm#latest-updates
https://arxiv.org/pdf/2608.23642
https://github.com/kgoedecke/doop
https://github.com/rawfilejson/awesome-osint-arsenal
https://github.com/CopilotKit/OpenBot
https://yoyobook.yolog.dev/#/en/title
https://yoyo-gasp.yolog.dev/
https://github.com/yologdev
https://github.com/yologdev/yoyo-gasp
https://github.com/yologdev/yoyo-evolve
https://github.com/yologdev/yopedia
https://github.com/yologdev/gasp
https://github.com/jason5ng32/MyIP
https://github.com/Graphify-Labs/graphify
https://github.com/ApodexAI/FrontierAgent
https://arxiv.org/abs/2608.24876
https://github.com/tt-a1i/simplify-codebase
https://github.com/Panniantong/Agent-Reach
https://github.com/haskaomni/blueprint
https://github.com/CarterPerez-dev/Cybersecurity-Projects
https://github.com/SenteLabsAI/OpenExecutive
https://github.com/openrung/openrung
https://github.com/songsummer920-dazzle/three-scope-map-skill
https://github.com/diegosouzapw/OmniRoute
https://arxiv.org/abs/2608.24876
https://github.com/Soulmate-Halo/qiling-soulmate
https://github.com/PDFMathTranslate/PDFMathTranslate
https://arxiv.org/abs/2608.00267
https://github.com/mco-org/squad
https://github.com/coleam00/excalidraw-diagram-skill
https://github.com/watercrawl/WaterCrawl
https://github.com/funstory-ai/BabelDOC
https://github.com/Manavarya09/design-extract
https://github.com/rustdesk/rustdesk
https://github.com/JetBrains/thinkrail
https://github.com/Osmantic/ODS
https://github.com/YouMind-OpenLab/ai-image-prompts-skill
https://github.com/whyubel1eve/promo-bgm-skill
https://github.com/Mr-funny/hbg-hanzi-chaizi-video
https://github.com/koala73/worldmonitor
https://arxiv.org/abs/2608.24876
https://github.com/ApodexAI/FrontierAgent
https://dive.antinomie.org/dsh-explore/#/cordis-from-dsh
https://github.com/LunarXuan/Pindo
https://github.com/baturyilmaz/wordpecker-app
https://www.alphaxiv.org/abs/2608.23566v2
https://github.com/IVRZ-da/agentiker-plan-follow
https://github.com/IVRZ-da/agentiker-code-intel
https://github.com/rarf/hermes-quota-plugin
https://github.com/kaishi00/hermes-community-plugins
https://github.com/theopitori/hermeskill
https://github.com/anthropics/claude-plugins-community/blob/main/eli5/skills/eli5/SKILL.md
https://github.com/humanlayer/skills/tree/main/plugins/show-me
https://github.com/warpdotdev/common-skills/tree/main/.agents/skills/skill-doctor
https://github.com/cursor/plugins/tree/main/pstack/skills/unslop
https://github.com/ApodexAI/FrontierAgent
https://www.alphaxiv.org/abs/2608.human-ai-collaboration-at-scalev1
https://github.com/moltlaunch/cashclaw
https://github.com/Chasen-Liao/pi-agent-desktop
https://rsi-exam.ai/blog.html
https://github.com/Vincentwei1021/video-shotcraft
https://github.com/NomaDamas/CozyClay
https://github.com/SamurAIGPT/Generative-Media-Skills
https://github.com/Panniantong/Agent-Reach
https://arxiv.org/abs/2409.16294
https://gencad.github.io/
https://github.com/ferdous-alam/GenCAD
https://huggingface.co/datasets/markov-ai/cad-1000-hours
https://github.com/roryclear/clearcam
https://github.com/video-db/call.md
https://github.com/ApodexAI/FrontierAgent
https://github.com/shanraisshan/claude-code-best-practice
https://hackers-arise.com/scada-ics-hacking-and-security-scada-protocols-and-their-purpose/
https://github.com/wuyoscar/GPT-Image2-Skill
https://github.com/itsOwen/CyberScraper-2077
https://arxiv.org/abs/2608.09696
https://arxiv.org/pdf/2608.22476
https://arxiv.org/abs/2608.23392
https://github.com/Straniero44/wenai
https://github.com/firecrawl/pdf-inspector
https://arxiv.org/abs/2608.23806
https://github.com/JuliusBrussee/caveman
https://github.com/farion1231/cc-switch
https://claude.com/blog/the-ai-native-sdlc-playbook
https://github.com/AgriciDaniel/claude-seo
https://github.com/ADScanPro/Claude-AD
https://github.com/calesthio/OpenMontage
https://github.com/yifanfeng97/hyper-extract
https://github.com/pavlobu/deskreen
https://github.com/gavinkhung/machine-learning-visualized
https://trendshift.io/
https://qingkeai.online/categories/jing-xuan-wen-zhang
https://github.com/GyulyVGC/sniffnet
https://github.com/ByteByteGoHq/system-design-101
https://github.com/browser-act/skills
https://github.com/ByteByteGoHq/system-design-101
https://github.com/Kodiqa-Solutions/VaultS3
https://github.com/ix-infrastructure/Ix
https://github.com/opentoonz/opentoonz
https://github.com/9527qingfeng/hantang-nihaixia-follower
https://github.com/earendil-works/pi/blob/137547a4/packages/agent/docs/agent-harness.md
https://github.com/JuliusBrussee/caveman
https://github.com/gnipbao/dao-skill
https://github.com/echohive42/AI-reads-books-page-by-page
https://github.com/lmcache/lmcache
https://github.com/kunchenguid/gnhf
https://github.com/Anil-matcha/Open-Generative-AI
https://www.perplexity.ai/hub/blog/brain-agentic-memory-as-a-knowledge-wiki
https://github.com/NousResearch/hermes-agent
https://github.com/liketrek/TREK
https://arxiv.org/abs/2608.20622
https://arxiv.org/abs/2608.19072
https://github.com/jonasstrehle/supercookie/
https://github.com/geeklee/srt-whiteboard-animation
https://github.com/Stirling-Tools/Stirling-PDF
https://github.com/tt-a1i/archify
https://github.com/ApodexAI/FrontierAgent
https://github.com/heygen-com/hyperframes
https://github.com/rainmanjam/poka-yoke
https://github.com/tashfeenahmed/freellmapi
https://github.com/crunz-ai/nativePDF-structurer
https://github.com/rawfilejson/awesome-osint-arsenal
https://github.com/garrytan/gstack
https://github.com/NVIDIA/SkillSpector
https://arxiv.org/abs/2608.19741
https://github.com/eli5-org/eli5
https://github.com/1N3/Sn1per
https://www.alphaxiv.org/abs/2608.23552
https://arxiv.org/pdf/2505.12540
https://github.com/viperrcrypto/Siftly
https://github.com/marc-shade/world-intel-mcp
https://paperswithcode.co/paper/2608.23552
https://github.com/microsoft/agent-lightning
https://github.com/huytieu/COG-second-brain
https://github.com/BraxisAI/braxis-blueprint
https://arxiv.org/abs/2608.19760
https://arxiv.org/abs/2608.22752
https://github.com/ix-infrastructure/Ix
https://github.com/odysseus-dev/odysseus
https://github.com/MengTo/sketchbook
https://paperswithcode.co/paper/2608.16812
https://github.com/tickernelz/opencode-mem
https://github.com/tt-a1i/archify
https://arxiv.org/abs/2608.20256
https://github.com/an2tha/onerep
https://github.com/kyegomez/OpenMythos
https://github.com/Alishahryar1/free-claude-code
https://arxiv.org/abs/2608.20845
https://github.com/YuJunZhiXue/dsh-purge
https://blog.kunchenguid.com/p/your-agentsmd-is-a-neural-net
https://arxiv.org/abs/2608.22118
https://github.com/handsomejustin/easy_tdx
https://github.com/HBAI-Ltd/Toonflow-app
https://github.com/henrythe9th/AI-Crash-Course
https://arxiv.org/abs/2608.17050
https://github.com/ripienaar/free-for-dev
https://github.com/topics/awesome-list
https://github.com/agentforce314/clawcodex
https://github.com/jacobpowaza/adaptive-zsh-completions
https://arxiv.org/abs/2601.23265
https://github.com/scanopy/scanopy
https://github.com/yibie/awesome-autoresearch
https://github.com/Yu9191/wloc
https://github.com/VoltAgent/awesome-agent-skills
https://github.com/K-Dense-AI/scientific-agent-skills
https://github.com/FrancescoStabile/numasec
https://github.com/apache/maka
https://github.com/coreyhaines31/marketingskills
https://github.com/rawfilejson/awesome-osint-arsenal
https://github.com/lidge-jun/opencodex
https://arxiv.org/abs/2404.07143
https://github.com/duty1g/x64dbg-mcp-server
https://github.com/jakubkrehel/skills
https://github.com/oomol-lab/pdf-craft
https://github.com/Hidashimora/free-vpn-anti-rkn
https://github.com/firstintent/ccteam
https://github.com/block/buzz
https://simonwillison.net/2026/Aug/19/conceptual-integrity-and-counting-lines-of-code/
https://www.turingpost.com/p/permanent-dawn
https://github.com/addyosmani/agent-skills/blob/main/skills/frontend-ui-engineering/SKILL.md
https://github.com/zhaoxuya520/reverse-skill
https://github.com/DEEP-JLU/Awesome-Graph-Engineering
https://github.com/JustVugg/colibri
https://www.aihero.dev/skills-resolving-merge-conflicts
https://github.com/AMAP-ML/LongHorizon-Harness
https://github.com/SANSAN0/TRENDRADAR
https://github.com/irbis-sh/zen-desktop
https://arxiv.org/abs/2605.29343
https://github.com/bilawalsidhu/gods-eye-view
https://github.com/microsoft/AI-Engineering-Coach
https://github.com/langchain-ai/openwiki
https://github.com/IceWhaleTech/CasaOS
https://github.com/xmanrui/dsh-im
https://github.com/b-nnett/grok-bot-0.18-reconstructed
https://github.com/laude-institute/headlong
https://github.com/memovai/mimimodel
https://github.com/Raphire/Win11Debloat
https://github.com/xszyou/Fay
https://github.com/lianghsun/open-sheet
https://github.com/KunAgent/Kun
https://github.com/nexu-io/open-design
https://github.com/DeusData/codebase-memory-mcp
https://github.com/jinzijian/EvoTrace
https://github.com/leoriczhang/teamEvolver
https://github.com/AMAP-ML/LongHorizon-Harness
https://github.com/colbymchenry/codegraph
https://github.com/addyosmani/agent-skills/blob/HEAD/docs/skill-anatomy.md
https://github.com/Gimanh/taskview-community
https://github.com/virattt/ai-hedge-fund
https://github.com/datalab-to/marker
https://github.com/Zyrexnn/Cybermes
https://arxiv.org/abs/2608.15191
https://github.com/router-for-me/CLIProxyAPI
https://arxiv.org/abs/2308.04512
https://github.com/projectdiscovery/nuclei-templates
https://github.com/firecrawl/anydoc
https://github.com/kuchin/awesome-ceo
https://surya.website/rling-qwen-to-paint-with-code
https://arxiv.org/abs/2608.18578
https://arxiv.org/abs/2608.18578
https://github.com/lencx/Minke
https://github.com/xbtlin/ai-berkshire
https://github.com/rohitg00/ai-engineering-from-scratch
https://github.com/MaxMiksa/Auto-Company
https://github.com/ripienaar/free-for-dev
https://arxiv.org/pdf/2602.17187
https://github.com/Leonxlnx/unlazy
https://github.com/ombharatiya/ai-system-design-guide/
https://github.com/b-nnett/grok-bot-0.18-reconstructed
https://github.com/EverMind-AI/SkillCorpus
https://github.com/42-evey/hermes-plugins
https://github.com/Romanescu11/hermes-skill-factory
https://github.com/asimons81/hermes-field-kit
https://github.com/CorsenAI/hermes-connector
https://github.com/DeployFaith/hermes-bible-skill
https://github.com/ai-boost/awesome-harness-engineering
https://github.com/dembrandt/dembrandt
https://github.com/furkankly/zoetrope
https://github.com/zhaoxuya520/reverse-skill
https://github.com/tjboudreaux/cc-thinking-skills
https://github.com/MkThingsHQ/mkagent
https://github.com/AgriciDaniel/claude-obsidian
https://www.alphaxiv.org/abs/2608.20169v1
https://github.com/citrolabs/ego-lite
"""

def classify_url(url: str) -> URLCategory:
    """根据URL内容分类"""
    url_lower = url.lower()
    for category, keywords in CATEGORY_KEYWORDS.items():
        for keyword in keywords:
            if keyword in url_lower:
                return category
    return URLCategory.UNKNOWN

def extract_github_info(url: str) -> Tuple[str, int]:
    """从GitHub URL提取项目名和stars"""
    parts = url.replace("https://github.com/", "").split("/")
    if len(parts) >= 2:
        repo = f"{parts[0]}/{parts[1]}"
        return repo, 0
    return url, 0

def extract_arxiv_id(url: str) -> str:
    """从arXiv URL提取论文ID"""
    match = re.search(r'(\d{4}\.\d{4,5})', url)
    if match:
        return match.group(1)
    return url

def create_kb_entry(url_info: URLInfo) -> Dict:
    """创建KB条目"""
    entry = {
        "action": "node:create",
        "title": url_info.title,
        "url": url_info.url,
        "node_type": "research",
        "tags": [url_info.category.value] + url_info.tags,
        "description": url_info.description,
        "metadata": {
            "source": "url_research_202609",
            "category": url_info.category.value,
            "stars": url_info.stars,
        }
    }
    return entry

def process_urls():
    """处理所有URL"""
    urls = [url.strip() for url in URLS_RAW.strip().split("\n") if url.strip()]
    
    print(f"📊 开始处理 {len(urls)} 个URL...")
    
    # 去重
    unique_urls = list(set(urls))
    print(f"✅ 去重后: {unique_urls} 个唯一URL")
    
    # 分类统计
    categories = {}
    for url in unique_urls:
        cat = classify_url(url)
        if cat not in categories:
            categories[cat] = []
        categories[cat].append(url)
    
    print("\n📈 URL分类统计:")
    for cat, urls in categories.items():
        print(f"  {cat.value}: {len(urls)} 个")
    
    # 准备批量导入
    batch_entries = []
    for url in unique_urls:
        cat = classify_url(url)
        
        if cat == URLCategory.GITHUB_REPO:
            repo, stars = extract_github_info(url)
            title = repo.split("/")[-1] if "/" in repo else repo
            desc = f"GitHub Repository: {repo}"
            tags = ["github", "repository"]
        elif cat == URLCategory.ARXIV_PAPER:
            paper_id = extract_arxiv_id(url)
            title = f"Paper: {paper_id}"
            desc = f"arXiv Paper: {paper_id}"
            tags = ["paper", "arxiv", "research"]
        elif cat == URLCategory.HUGGINGFACE:
            title = url.split("/")[-1]
            desc = f"HuggingFace Resource: {title}"
            tags = ["huggingface", "model", "dataset"]
        else:
            title = url.split("/")[-2] if "/" in url else url
            desc = f"Web Resource: {url}"
            tags = ["web", "resource"]
        
        entry = create_kb_entry(URLInfo(
            url=url,
            category=cat,
            title=title,
            description=desc,
            stars=stars,
            tags=tags
        ))
        batch_entries.append(entry)
    
    # 写入KB
    print(f"\n💾 写入KB...")
    success_count = 0
    for entry in batch_entries:
        try:
            json_str = json.dumps(entry, ensure_ascii=False)
            cmd = ["neotrix", "memory", "kb", "write", json_str, "--force"]
            result = subprocess.run(cmd, capture_output=True, text=True, cwd="/Users/neo/Downloads/neotrix")
            if result.returncode == 0:
                success_count += 1
            else:
                print(f"❌ 失败: {entry['title'][:30]}... - {result.stderr[:50]}")
        except Exception as e:
            print(f"❌ 异常: {entry['title'][:30]}... - {str(e)[:50]}")
    
    print(f"\n✅ 完成: {success_count}/{len(batch_entries)} 条目成功导入")
    
    # 返回统计信息
    return {
        "total": len(unique_urls),
        "by_category": {cat.value: len(urls) for cat, urls in categories.items()},
        "imported": success_count
    }

if __name__ == "__main__":
    stats = process_urls()
    print("\n📊 最终统计:", json.dumps(stats, indent=2))
