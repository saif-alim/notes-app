# Add a New iOS Screen

**Scope:** Add a new SwiftUI view + viewmodel to the iOS app.

## Steps

1. **Create ViewModel in `apps/ios/Sources/Features/<Feature>/`**
   ```swift
   @Observable
   final class <Feature>ViewModel {
       var state: <Feature>State = .loading
       let apiClient: APIClient
       
       func load() async {
           // ...
       }
   }
   ```

2. **Create SwiftUI View in same directory**
   ```swift
   struct <Feature>View: View {
       @State var viewModel: <Feature>ViewModel
       
       var body: some View {
           // ...
       }
   }
   ```

3. **Wire into App.swift navigation** (app router)

4. **Add XCTest in `apps/ios/Tests/`**
   - Test ViewModel state transitions
   - Mock APIClient

5. **Update `apps/ios/CLAUDE.md`** with new screen's location and model

See `apps/ios/CLAUDE.md` (Phase 7+) for detailed patterns.
