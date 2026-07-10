#!/usr/bin/env python3
import re

# Read the file
with open('src/neotrix/reasoning_brain/tests.rs', 'r') as f:
    content = f.read()

# Fix 1: Add micro_edits creation after each SelfEdit definition
# Pattern: let edit = ...; (with SelfEdit)
# Add: let micro_edits = vec![MicroEdit::AdjustDimension("typography".to_string(), 0.1)];

# Find all SelfEdit blocks and add micro_edits after them
self_edit_pattern = r'(let edit = crate::neotrix::reasoning_brain::SelfEdit \{[^}]+\});'
micro_edits_line = '\n        let micro_edits = vec![MicroEdit::AdjustDimension("typography".to_string(), 0.1)];'

def add_micro_edits(match):
    return match.group(0) + micro_edits_line

content = re.sub(self_edit_pattern, add_micro_edits, content, flags=re.DOTALL)

# Fix 2: Replace ReasoningMemory::new("...", &edit, 0.X) with ReasoningMemory::new("...", TaskType::UIDesign, &micro_edits, 0.X)
content = re.sub(
    r'ReasoningMemory::new\("([^"]+)", &edit, ([\d.]+)\)',
    r'ReasoningMemory::new("\1", TaskType::UIDesign, &micro_edits, \2)',
    content
)

# Fix 3: Replace retrieve_relevant_by_embedding(&vec![...], N) with retrieve_relevant_by_embedding(&vec![...], None, N)
content = re.sub(
    r'retrieve_relevant_by_embedding\((&vec!\[[^\]]+\]), (\d+)\)',
    r'retrieve_relevant_by_embedding(\1, None, \2)',
    content
)

# Fix 4: Replace retrieve_relevant("...", N) with retrieve_relevant("...", None, N)
content = re.sub(
    r'retrieve_relevant\("([^"]+)", (\d+)\)',
    r'retrieve_relevant("\1", None, \2)',
    content
)

# Fix 5: Fix assert_eq! with missing comma (if any)
# The pattern assert_eq!(stats.total_memories, N) should be correct, but let's check
content = re.sub(
    r'assert_eq!\(stats\.total_memories, (\d+)\)',
    r'assert_eq!(stats.total_memories, \1)',
    content
)

# Write back
with open('src/neotrix/reasoning_brain/tests.rs', 'w') as f:
    f.write(content)

print("Done! Fixed tests.rs")
