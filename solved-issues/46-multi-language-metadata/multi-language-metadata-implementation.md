# Multi-Language Metadata Implementation

## Overview

This implementation adds support for multi-language course metadata in the Stream-Scholar contracts, allowing courses to store IPFS links for different language versions of the same course content.

## Features

### 1. Multi-Language Metadata Structure
- **CourseMetadata**: Main structure containing course-wide language settings
- **LanguageMetadata**: Individual language-specific metadata with IPFS links
- Support for ISO 639-1 language codes (e.g., "en", "es", "zh", "fr")
- Default language configuration
- Available languages tracking

### 2. IPFS Integration
- Each language version stores its own IPFS CID for metadata JSON
- Optional language-specific thumbnail IPFS CIDs
- Base metadata CID for fallback content
- Decentralized storage for all language variants

### 3. Smart Contract Functions

#### Core Functions
- `create_course_metadata()` - Initialize course with default language
- `add_language_metadata()` - Add new language version
- `update_language_metadata()` - Update existing language version
- `remove_language_metadata()` - Remove language version

#### Query Functions
- `get_course_metadata()` - Get course metadata info
- `get_language_metadata()` - Get specific language metadata
- `get_available_languages()` - List all available languages
- `get_default_language()` - Get current default language

#### Management Functions
- `set_default_language()` - Change default language
- Authorization controls (admin/course creator only)

## JSON Schema Updates

The schema now includes a `multi_language_metadata` object with:

```json
{
  "multi_language_metadata": {
    "default_language": "en",
    "available_languages": ["en", "es", "zh", "fr"],
    "metadata_links": {
      "en": {
        "ipfs_cid": "QmEn123...",
        "title": "Course Title",
        "description": "Course description",
        "thumbnail": {
          "ipfs_cid": "QmEnThumb...",
          "mimeType": "image/webp"
        },
        "learning_objectives": ["Objective 1", "Objective 2"],
        "last_updated": "2024-03-25T10:00:00Z"
      },
      "es": {
        "ipfs_cid": "QmEs456...",
        "title": "Título del Curso",
        "description": "Descripción del curso",
        // ... other Spanish-specific fields
      }
    }
  }
}
```

## Usage Examples

### Creating Course with Multi-Language Support

```rust
// Initialize course with English as default
contract.create_course_metadata(
    course_id: 1,
    default_language: "en",
    base_metadata_cid: "QmBase123...",
    creator: instructor_address
);

// Add Spanish version
contract.add_language_metadata(
    course_id: 1,
    language_code: "es",
    ipfs_cid: "QmSpanish456...",
    title: "Curso de Ejemplo",
    description: "Descripción en español",
    thumbnail_cid: Some("QmSpanishThumb..."),
    creator: instructor_address
);

// Add Chinese version
contract.add_language_metadata(
    course_id: 1,
    language_code: "zh",
    ipfs_cid: "QmChinese789...",
    title: "课程标题",
    description: "课程描述",
    thumbnail_cid: Some("QmChineseThumb..."),
    creator: instructor_address
);
```

### Querying Language Metadata

```rust
// Get available languages
let languages = contract.get_available_languages(course_id);
// Returns: ["en", "es", "zh"]

// Get Spanish metadata
let spanish_metadata = contract.get_language_metadata(course_id, "es");
// Access title, description, IPFS CID, etc.

// Get current default language
let default_lang = contract.get_default_language(course_id);
```

### Managing Languages

```rust
// Change default language to Spanish
contract.set_default_language(course_id, "es", creator);

// Update Spanish metadata
contract.update_language_metadata(
    course_id,
    "es",
    "QmSpanishUpdated...",
    "Título Actualizado",
    "Descripción actualizada",
    Some("QmNewThumb..."),
    creator
);

// Remove French language (if not default)
contract.remove_language_metadata(course_id, "fr", creator);
```

## Security and Authorization

### Access Control
- **Admin**: Full access to all language metadata operations
- **Course Creator**: Can manage metadata for their own courses
- **Public**: Read-only access to query functions

### Validation
- Language codes must be valid ISO 639-1 (2-letter codes)
- Cannot remove default language
- Cannot add duplicate language versions
- IPFS CID validation for proper format

### Events
The contract emits events for all metadata operations:
- `Course_Metadata_Created`
- `Language_Metadata_Added`
- `Language_Metadata_Updated`
- `Language_Metadata_Removed`
- `Default_Language_Updated`

## Testing

Comprehensive test suite covering:
- ✅ Basic CRUD operations
- ✅ Authorization controls
- ✅ Language management
- ✅ Default language handling
- ✅ Error conditions
- ✅ Edge cases

Run tests with:
```bash
cargo test --lib test_create_course_metadata
cargo test --lib test_add_language_metadata
cargo test --lib test_update_language_metadata
# ... etc
```

## Storage Optimization

### TTL Management
- All metadata entries use automatic TTL extension
- Prevents data expiration while maintaining efficiency
- Configurable bump thresholds and extensions

### Storage Layout
- `CourseMetadata(course_id)` - Main course metadata
- `CourseLanguageMetadata(course_id, language_code)` - Language-specific data
- Efficient lookup patterns for common queries

## Integration Considerations

### Frontend Integration
1. Query available languages for a course
2. Fetch user's preferred language
3. Fall back to default language if preferred not available
4. Load language-specific metadata from IPFS

### IPFS Structure
Recommended IPFS directory structure:
```
/course-metadata/
├── {course_id}/
│   ├── base.json (fallback metadata)
│   ├── en.json (English metadata)
│   ├── es.json (Spanish metadata)
│   ├── zh.json (Chinese metadata)
│   └── thumbnails/
│       ├── en.jpg
│       ├── es.jpg
│       └── zh.jpg
```

## Future Enhancements

### Potential Improvements
1. **Automatic Translation**: Integration with translation services
2. **Language Detection**: Auto-detect user's preferred language
3. **Content Versioning**: Track versions per language
4. **Bulk Operations**: Add/update multiple languages at once
5. **Language Statistics**: Track usage per language

### Scalability
- Support for unlimited languages per course
- Efficient gas usage for language operations
- Optimized storage patterns for high-volume usage

## Conclusion

This implementation provides a robust, scalable solution for multi-language course metadata in the Stream-Scholar platform. It maintains backward compatibility while adding powerful new internationalization capabilities through decentralized IPFS storage.

The design prioritizes:
- **Security**: Proper authorization and validation
- **Efficiency**: Optimized storage and gas usage
- **Flexibility**: Support for any number of languages
- **Usability**: Simple API for frontend integration
- **Decentralization**: IPFS-based storage for resilience
