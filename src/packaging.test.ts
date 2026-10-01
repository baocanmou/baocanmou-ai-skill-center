import { describe, expect, it } from 'vitest'
import { scripts } from '../package.json'
import cargo from '../src-tauri/Cargo.toml?raw'

describe('Desktop packaging', () => {
  it('embeds production assets instead of depending on the Vite development server', () => {
    expect(cargo).toMatch(/\[features\]\s+custom-protocol\s*=\s*\["tauri\/custom-protocol"\]/)
    expect(scripts['tauri:build']).toContain('--features custom-protocol')
  })
})
