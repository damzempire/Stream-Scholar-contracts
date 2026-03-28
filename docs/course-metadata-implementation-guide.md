# Course Metadata Standard Implementation Guide

## Overview

This guide explains how to implement and use the Course Metadata Standard JSON schema for Stream Scholar contracts. The standard defines a consistent format for course descriptions, thumbnails, and durations to be stored on IPFS.

## Schema Location

The JSON schema is located at: `docs/course-metadata-standard.json`

## Key Features

### Core Fields
- **courseId**: Unique identifier (1-64 characters, alphanumeric + underscore/hyphen)
- **title**: Course title (1-200 characters)
- **description**: Detailed course description (1-2000 characters)
- **instructor**: Instructor object with name, wallet address, and optional bio/avatar
- **duration**: Duration object with total minutes and calculated hours
- **thumbnail**: Thumbnail object with IPFS CID, MIME type, and metadata
- **createdAt/updatedAt**: ISO 8601 timestamps

### Optional Fields
- **category**: Primary/secondary categories and tags
- **difficulty**: beginner, intermediate, advanced, expert
- **language**: ISO 639-1 language code (default: "en")
- **price**: Pricing information with currency and amount
- **prerequisites**: Array of prerequisite course IDs
- **learningObjectives**: Array of learning objectives
- **version**: Schema version (default: "1.0.0")
- **status**: Publication status (draft, published, archived, deprecated)

## IPFS Integration

### Thumbnail Storage
Thumbnails should be uploaded to IPFS and referenced using the CID in the metadata:

```json
{
  "thumbnail": {
    "ipfsCid": "QmYyy... (actual IPFS CID)",
    "mimeType": "image/jpeg",
    "dimensions": {
      "width": 1280,
      "height": 720
    },
    "fileSize": 245760
  }
}
```

### CID Validation
The schema validates IPFS CIDs using these patterns:
- IPFS v0: `^Qm[1-9A-HJ-NP-Za-km-z]{44,}$`
- IPFS v1: `^b[A-Za-z2-7]{58,}$`

## Usage Examples

### Basic Course Metadata
```json
{
  "courseId": "intro-to-blockchain-101",
  "title": "Introduction to Blockchain",
  "description": "Learn blockchain fundamentals and distributed ledger technology.",
  "instructor": {
    "name": "Bob Smith",
    "address": "GD5JJ3STX5U5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5"
  },
  "duration": {
    "totalMinutes": 300,
    "estimatedHours": 5,
    "format": "hours"
  },
  "thumbnail": {
    "ipfsCid": "QmXxx...",
    "mimeType": "image/png"
  },
  "createdAt": "2024-03-25T10:00:00Z",
  "updatedAt": "2024-03-25T10:00:00Z"
}
```

### Advanced Course Metadata
```json
{
  "courseId": "advanced-smart-contracts-301",
  "title": "Advanced Smart Contract Development",
  "description": "Deep dive into advanced smart contract patterns and security best practices.",
  "instructor": {
    "name": "Carol Davis",
    "address": "GD5JJ3STX5U5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5XK5",
    "bio": "Blockchain security expert with 15+ years experience",
    "avatar": "QmZzz..."
  },
  "duration": {
    "totalMinutes": 720,
    "estimatedHours": 12,
    "format": "hours"
  },
  "thumbnail": {
    "ipfsCid": "QmAaa...",
    "mimeType": "image/jpeg",
    "dimensions": {
      "width": 1920,
      "height": 1080
    },
    "fileSize": 512000
  },
  "category": {
    "primary": "blockchain",
    "secondary": "programming",
    "tags": ["smart-contracts", "security", "solidity", "rust"]
  },
  "difficulty": "advanced",
  "language": "en",
  "price": {
    "amount": "50000000",
    "currency": "XLM",
    "isFree": false
  },
  "prerequisites": ["intro-to-blockchain-101", "basic-programming-101"],
  "learningObjectives": [
    "Design secure smart contract architectures",
    "Implement advanced DeFi patterns",
    "Audit and test smart contracts effectively"
  ],
  "createdAt": "2024-03-25T10:00:00Z",
  "updatedAt": "2024-03-25T10:00:00Z",
  "version": "1.0.0",
  "status": "published"
}
```

## Validation

### JSON Schema Validation
Use a JSON schema validator to ensure compliance:

```javascript
const Ajv = require('ajv');
const schema = require('./course-metadata-standard.json');

const ajv = new Ajv();
const validate = ajv.compile(schema);

const valid = validate(courseMetadata);
if (!valid) {
  console.log(validate.errors);
}
```

### Required Field Validation
Ensure all required fields are present:
- courseId
- title
- description
- instructor (with name and address)
- duration (with totalMinutes)
- thumbnail (with ipfsCid)
- createdAt
- updatedAt

## Integration with Smart Contracts

### Storing Metadata CID
Store the IPFS CID of the course metadata in the smart contract:

```rust
// In your Soroban contract
fn store_course_metadata(env: &Env, course_id: &String, metadata_cid: &BytesN<32>) {
    // Store the CID mapping
    let courses = env.storage().instance();
    courses.set(course_id, metadata_cid);
}

fn get_course_metadata(env: &Env, course_id: &String) -> Option<BytesN<32>> {
    let courses = env.storage().instance();
    courses.get(course_id)
}
```

### Frontend Integration
Fetch and parse metadata from IPFS:

```javascript
async function getCourseMetadata(courseId, contract) {
  const metadataCid = await contract.get_course_metadata(courseId);
  const metadata = await fetchFromIPFS(metadataCid);
  return JSON.parse(metadata);
}
```

## Best Practices

1. **Image Optimization**: Compress thumbnails to < 10MB before IPFS upload
2. **Consistent Timestamps**: Use UTC timestamps in ISO 8601 format
3. **Version Control**: Update the version field when making schema changes
4. **Validation**: Always validate metadata before storing on IPFS
5. **Backup**: Keep local backups of important metadata

## Migration Guide

When upgrading from a previous metadata format:

1. Map existing fields to the new schema
2. Add missing required fields with sensible defaults
3. Validate against the new schema
4. Update IPFS references in smart contracts
5. Update frontend parsing logic

## Support

For questions or issues with the Course Metadata Standard:
- Create an issue in the repository
- Check the examples in the schema file
- Review the validation patterns
