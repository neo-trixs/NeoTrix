# Star Valley Art Style Guide

## Color Palette

### Primary Colors
- **Forest Green**: #2d5a3d (grass, nature)
- **Earth Brown**: #5d4037 (dirt, wood)
- **Sky Blue**: #42a5f5 (water, UI accents)
- **Sunset Gold**: #f4d03f (highlights, important items)
- **Deep Night**: #1a1a2e (backgrounds)

### Secondary Colors
- **Leaf Green**: #4caf50 (crops, vegetation)
- **Stone Gray**: #757575 (rocks, minerals)
- **Warm Wood**: #8d6e63 (fences, buildings)
- **Soft Pink**: #f8bbd9 (flowers, NPCs)
- **Rich Purple**: #7c4dff (magic, rare items)

### UI Colors
- **Panel Background**: rgba(0,0,0,0.7)
- **Panel Border**: #8b6914 (wooden frame)
- **Text Primary**: #ffffff
- **Text Secondary**: #aaaaaa
- **Success**: #4caf50
- **Warning**: #ff9800
- **Danger**: #f44336

## Typography

### Headings
- Font: 'Courier New', monospace (retro game feel)
- Weight: bold
- Color: #f4d03f (gold)
- Shadow: 2px 2px 4px rgba(0,0,0,0.5)

### Body Text
- Font: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif
- Size: 12-14px
- Color: #ffffff
- Line height: 1.5

### UI Labels
- Font: 'Courier New', monospace
- Size: 10-12px
- Color: #aaaaaa
- Letter spacing: 1px

## Visual Elements

### Player Character
- Simple geometric shape (circle with hat)
- Gold color (#f4d03f) for visibility
- Smooth movement animation
- Tool indicator on right side

### NPCs
- Colored circles with names
- Each has unique color:
  - Awareness: #4FC3F7 (light blue)
  - Focus: #EF5350 (red)
  - Creativity: #FFD54F (yellow)
  - Empathy: #66BB6A (green)
  - Memory: #AB47BC (purple)
  - Logic: #5C6BC0 (indigo)
  - Wisdom: #FFA726 (orange)
  - Dreams: #EC407A (pink)

### Crops
- Start as small seeds
- Grow in 4 stages: seed → sprout → growing → ready
- Color indicates type
- Golden glow when ready to harvest

### Tiles
- **Grass**: Two-tone green pattern
- **Dirt**: Solid brown
- **Water**: Blue with wave animation
- **Trees**: Green circle on brown trunk
- **Rocks**: Gray circles
- **Paths**: Light brown
- **Farm Plots**: Dark brown with border

## Animation Guidelines

### Movement
- Smooth interpolation (not instant)
- Speed: 3 pixels per frame
- Camera follows player with slight lag

### Effects
- Particles for all interactions
- Screen shake for impacts
- Floating text for feedback
- Glow effects for important items

### Transitions
- Fade in/out for menus (0.3s)
- Slide in for panels (0.2s)
- Scale bounce for buttons

## Sound Design

### Ambient
- Wind in trees
- Water flowing
- Birds chirping
- Night crickets

### Actions
- Footsteps (soft thuds)
- Tool use (distinct per tool)
- Harvest (satisfying pop)
- Coin (ching sound)
- Level up (triumphant fanfare)

### UI
- Menu open (soft click)
- Menu close (lower click)
- Button hover (subtle tone)
- Error (buzz sound)

## Layout Principles

### HUD
- Top: Time, Season, Health, Energy, Gold
- Bottom: Tool bar
- Right: Quest tracker
- Left: Minimap

### Menus
- Centered panels with wooden borders
- Semi-transparent background
- Clear hierarchy with headings
- Consistent spacing

### Dialogue
- Bottom of screen
- NPC name in color
- Text appears gradually
- Choices highlighted on hover

## Performance Tips

### Rendering
- Only draw visible tiles
- Use object pooling for particles
- Batch similar draw calls
- Cache frequently used calculations

### Memory
- Reuse objects instead of creating new ones
- Limit particle count
- Clean up unused references
