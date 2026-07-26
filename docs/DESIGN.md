---
name: KeyVault
description: Zero-knowledge, local-first password manager with a Rust cryptographic core.
colors:
  primary: "#fafafa"
  primary-foreground: "#18181b"
  background: "#09090b"
  foreground: "#fafafa"
  secondary: "#27272a"
  secondary-foreground: "#fafafa"
  muted-foreground: "#a1a1aa"
  destructive: "#7f1d1d"
  destructive-foreground: "#fafafa"
  border: "#27272a"
  ring: "#d4d4d8"
  accent: "#06b6d4"
typography:
  display:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "-0.025em"
  headline:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif"
    fontSize: "14px"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "-0.02em"
  title:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif"
    fontSize: "13px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "normal"
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif"
    fontSize: "12px"
    fontWeight: 600
    lineHeight: 1.0
    letterSpacing: "0.05em"
  mono:
    fontFamily: "monospace"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0.02em"
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
  xxl: "32px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.primary-foreground}"
    rounded: "{rounded.lg}"
    padding: "8px 10px"
  button-primary-hover:
    backgroundColor: "rgba(250, 250, 250, 0.8)"
  button-secondary:
    backgroundColor: "{colors.secondary}"
    textColor: "{colors.secondary-foreground}"
    rounded: "{rounded.lg}"
    padding: "8px 10px"
  button-secondary-hover:
    backgroundColor: "rgba(39, 39, 42, 0.8)"
  input-text:
    backgroundColor: "transparent"
    textColor: "{colors.foreground}"
    rounded: "{rounded.lg}"
    padding: "4px 10px"
---

# Design System: KeyVault

This document outlines the visual identity, styling specifications, and core user experience principles for **KeyVault**. Every interface (the browser extension popup, options page, dialogs, and components) must align with this system.

---

## 1. Design Philosophy

**Creative North Star: "The Charcoal Vault"**

The visual language of KeyVault is guided by "The Charcoal Vault": tactile dark charcoal surfaces, clean utility, and zero visual noise. Designed to serve as a highly secure browser extension, it eschews any light themes or distracting visual elements in favor of a cohesive, high-contrast dark palette.

Depth is flat-by-default, established purely through subtle value shifts between dark charcoal UI elements (`#18181b` / `#27272a`) and a deep neutral background (`#09090b`). Visual density is high but clear, allowing the interface to disappear into the security workflows. Consistent ShadCN components enforce standard web actions so the vault feels familiar, safe, and highly precise.

- **Utilitarian Clarity**: Information hierarchy is absolute. The tool disappears into the task.
- **Flat Elevation**: Depth is conveyed through subtle tonal shifts, not drop shadows.
- **Restrained Accents**: Accents are used transitionally and sparingly (≤10% of any screen) to highlight active workflows.

---

## 2. Color System

KeyVault uses a strictly dark, low-saturation charcoal color palette optimized for high-contrast accessibility. Colors are defined in hex, HSL, and OKLCH.

| Token | Hex | HSL | OKLCH | Primary Application |
|---|---|---|---|---|
| **Deep Background** | `#09090b` | `240 10% 3.9%` | `oklch(0.12 0.005 240)` | Outer panel backgrounds |
| **Charcoal Container** | `#18181b` | `240 5.9% 10%` | `oklch(0.18 0.006 240)` | Nested cards, panels, tab bars |
| **Dark Charcoal** | `#27272a` | `240 3.7% 15.9%` | `oklch(0.24 0.006 240)` | Form borders, secondary buttons |
| **Charcoal White** | `#fafafa` | `0 0% 98%` | `oklch(0.98 0.001 0)` | Primary text, primary CTA buttons |
| **Slate Muted** | `#a1a1aa` | `240 5% 64.9%` | `oklch(0.70 0.010 240)` | Secondary text, placeholders |
| **Electric Steel Blue** | `#06b6d4` | `188 95% 43%` | `oklch(0.72 0.170 195)` | Active rings, links, interactive highlights |
| **Security Red** | `#7f1d1d` | `0 62.8% 30.6%` | `oklch(0.28 0.100 20)` | Destructive operations (Delete, Reset, Purge) |

### Accent Allocation Rule
- **The 10% Rule**: Primary White and Electric Steel Blue must not exceed 10% of any viewport's surface area. Their primary purpose is to draw the eye to critical user targets.

---

## 3. Spacing & Rhythm

All spacing must adhere to a strict linear 4px unit grid. Random spacing values (e.g., `11px`, `13px`) are forbidden.

- **`xs` (4px)**: Tiny spacing, inline elements, badge padding.
- **`sm` (8px)**: Tight gaps between adjacent fields, title to subtitle gaps.
- **`md` (12px)**: Default spacing inside list items and small containers.
- **`lg` (16px)**: Default outer container padding for extension popups, card padding.
- **`xl` (24px)**: Page padding for widescreen options panel sections.
- **`xxl` (32px)**: Generous header spacing or section divides.

---

## 4. Typography

KeyVault uses a single, well-tuned sans-serif system stack to eliminate decorative noise and ensure fast performance.

```css
font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
```

### Type Hierarchy

| Level | Weight | Size | Line Height | Letter Spacing | Application |
|---|---|---|---|---|---|
| **Display** | Bold (700) | `15px` | `1.2` | `-0.025em` | App branding, main headers |
| **Headline** | Bold (700) | `14px` | `1.2` | `-0.02em` | Form header titles |
| **Title** | Semi-Bold (600) | `13px` | `1.3` | `normal` | Credential lists titles |
| **Body** | Regular (400) | `14px` | `1.5` | `normal` | Text fields, notes, paragraphs |
| **Label** | Semi-Bold (600) | `12px` | `1.0` | `0.05em` | Eyebrow uppercase labels |
| **Mono** | Regular (400) | `12px` | `1.4` | `0.02em` | Password generation read-outs |

---

## 5. Interaction States & Transitions

All interactive components must support all states cleanly and transition smoothly.

### Focus Rings
- Focused inputs and buttons must transition their border to Electric Steel Blue (`var(--ring)`) with a subtle active focus ring.
- Focus outline overrides must always be replaced with a visible high-contrast ring.

### Copy Feedback
- Copy buttons must display a checkmark (`Check` icon) colored in green (`text-green-400`) for exactly **2 seconds** before returning to the default state.

### State Transitions
- **Easing**: Exponential ease-out (`cubic-bezier(0.16, 1, 0.3, 1)`).
- **Duration**: `150ms` for micro-interactions (hover, focus), `250ms` for panel entry and tabs.
- **Reduced Motion**: Respect `@media (prefers-reduced-motion: reduce)` by disabling scaling/sliding transforms and fallback to instant style updates or crossfades.

---

## 6. Layout Systems

### 1. Extension Popup (`360px × 500px`)
- Strictly constrained dimensions.
- High vertical layout density.
- Scrollable list containers limited to a maximum height of `280px` to prevent overflow of bottom action buttons.
- Bottom actions are fixed or sticky at the bottom.

### 2. Widescreen Dashboard (Options Page)
- **Grid Layout**: Dual-column layout. 
  - Left pane (`320px` width) for search and listings.
  - Right pane (`grow`) for detail cards, form creation, and configurations.
  - Sidebar collapses or reflows on small viewports.

---

## 7. Component Specifications

### Buttons
- **Shape**: Rounded corners with an `8px` (`0.5rem`) radius.
- **Primary CTA**: Background `#fafafa`, text `#18181b`. Hover transitions opacity to `90%`.
- **Secondary**: Background `#27272a`, text `#fafafa`. Hover shifts background to `#3f3f46`.
- **Destructive**: Background `#7f1d1d` (low opacity `bg-destructive/10` or `bg-destructive/20`), text `#ef4444`.

### Cards & Container Borders
- Standard borders are `1px solid #27272a`.
- Shadows are forbidden for flat containers. Shadows are allowed only on floating popovers or dropdown menus.

### Inputs / Fields (ShadCN Input)
- **Style**: Background `#09090b`, border 1px solid `#27272a`, radius 8px.
- **Focus**: Border transitions to `#d4d4d8` with a subtle active ring.

### Tabs (ShadCN Tabs)
- **Root List**: Background `#18181b`, border 1px solid `#27272a`, padding 2px.
- **Active Trigger**: Background `#27272a`, text `#fafafa`.

---

## 8. Do's and Don'ts

### Do:
- **Do** maintain a strict 4.5:1 contrast ratio for all readable text.
- **Do** disable or simplify transitions when `@media (prefers-reduced-motion: reduce)` is active.
- **Do** use exact 8px rounded corners for primary buttons and container cards.
- **Do** use standard ShadCN-Svelte component classes and wrappers.

### Don't:
- **Don't** use side-stripe borders as accents on lists or alert boxes.
- **Don't** pair 1px borders with wide, blurry drop shadows on cards.
- **Don't** use neon or saturated color gradients on text.
- **Don't** use border-radius values larger than 12px for cards or inputs.
- **Don't** introduce light theme surfaces or backgrounds.
