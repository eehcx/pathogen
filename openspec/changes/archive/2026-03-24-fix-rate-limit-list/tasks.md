# Tasks: Fix RateLimitList Rendering and State Bug

## Phase 1: Fix State Clone Bug

- [x] 1.1 Fix state clone in `src/presentation/views/rate_limit_list.rs` line 89 - change `&mut app.rate_limit_list.state.clone()` to `&mut app.rate_limit_list.state` (direct mutable reference)

## Phase 2: Add Missing Render Call

- [x] 2.1 Add missing `rate_limit_list::render_rate_limit_list(frame, app, chunks[1]);` call in `src/presentation/ui.rs` for `AppMode::RateLimitList` case (after header, before footer)

## Phase 3: Verification

- [ ] 3.1 Verify RateLimitList renders correctly by building and running the application
- [ ] 3.2 Verify selection persists when navigating away and back to RateLimitList