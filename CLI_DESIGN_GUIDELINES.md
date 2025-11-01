# CLI Design Guidelines

## Command Line Interface Style

### Recommended: Cisco-Style CLI

The Q CLI should adopt a Cisco-style command line interface instead of the current bash-style approach for better user experience and discoverability.

#### Current Implementation (Bash-Style)
```bash
q skills --help
q skills list --verbose
q skills create --name test-skill --type code_inline
```

#### Recommended Implementation (Cisco-Style)
```bash
q skills ?
q skills list ?
q skills create test-skill code_inline
```

### Key Differences

| Feature | Bash-Style (Current) | Cisco-Style (Recommended) |
|---------|---------------------|---------------------------|
| Help | `--help`, `-h` | `?` |
| Verbose | `--verbose`, `-v` | `verbose` (keyword) |
| Parameters | `--name value` | `value` (positional) |
| Completion | Tab completion | `?` shows options |
| Discovery | Must know flags | Context-sensitive help |

### Benefits of Cisco-Style CLI

1. **Intuitive Help**: `?` at any point shows available options
2. **Context-Aware**: Help changes based on current command context
3. **Faster Input**: No need for `--` prefixes on common operations
4. **Better Discovery**: Users can explore commands naturally
5. **Consistent**: Matches network equipment and enterprise tools

### Implementation Examples

#### Help System
```bash
# Current
q skills --help

# Recommended
q skills ?
Available commands:
  list     Show available skills
  create   Create new skill
  run      Execute skill
  info     Show skill details
```

#### Command Completion
```bash
# Current
q skills create --name <TAB>

# Recommended  
q skills create ?
<skill-name> <skill-type>
  skill-name: Name for the new skill
  skill-type: code_inline | file_based | interactive
```

#### Parameter Input
```bash
# Current
q skills run --skill calculator --input "2+2"

# Recommended
q skills run calculator "2+2"
```

### Migration Strategy

1. **Phase 1**: Add `?` help alongside existing `--help`
2. **Phase 2**: Implement context-sensitive help system
3. **Phase 3**: Add positional parameter support
4. **Phase 4**: Deprecate bash-style flags (with warnings)
5. **Phase 5**: Remove bash-style support

### Technical Implementation

- Use `clap` derive macros with custom help formatting
- Implement `?` as special input handler
- Add context-aware completion engine
- Create command tree for hierarchical help

### User Experience Impact

- **Learning Curve**: Reduced - more intuitive for new users
- **Efficiency**: Improved - faster command entry
- **Discoverability**: Enhanced - built-in exploration
- **Consistency**: Better - matches enterprise tool expectations

This design change would significantly improve the Q CLI user experience by making it more discoverable and efficient to use.
