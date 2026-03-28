# Stream Scholar SDK Wrapper

This SDK provides a simplified JavaScript interface for the Stream Scholar contract.

## Abstractions

- `startStream()` → wraps `buy_access`
- `stopStream()` → wraps `pro_rated_refund`
- `sendHeartbeat()` → wraps `heartbeat`

## Example

```ts
import { StreamScholarSDK } from "./streamScholarSdk";

const sdk = new StreamScholarSDK(client);

await sdk.startStream({
  student: "G...",
  courseId: 1,
  amount: "1000",
  token: "G...",
});

await sdk.sendHeartbeat({
  student: "G...",
  courseId: 1,
  signature: "test_signature",
});

await sdk.stopStream({
  student: "G...",
  courseId: 1,
});