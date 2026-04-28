# Stream-Scholar Student Error Guide

This document maps protocol-level error codes to helpful explanations for students and developers.

## Error Code Mapping

| Code | Name | Description | Student-Friendly Advice |
| :--- | :--- | :--- | :--- |
| **101** | `Deposit_Below_Minimum` | The deposit amount is less than the protocol's required minimum. | "The amount you're trying to fund is too low. Please check the minimum deposit requirement for this course." |
| **102** | `Heartbeat_Too_Frequent` | Tracking updates are sent more often than allowed. | "You're sending updates too quickly. Please wait a few moments before continuing your session." |
| **103** | `Access_Expired` | Your funded access time for this course has ended. | "Your access has expired. Please top up your scholarship or purchase more airtime to continue." |
| **104** | `Invalid_Rate` | The dynamic rate calculation failed or returned zero. | "There was a calculation error with your stream rate. Please contact support or your institution." |
| **205** | `Probation_Active_Withdrawal_Limited` | Withdrawal is limited due to a probation period or unverified academic performance. | "Your withdrawal is limited because your latest GPA update was below 2.0 or academic verification is pending." |
| **301** | `Emergency_Stop_Active` | A technical auditor has paused all academic streams for a security review. | "The platform is currently undergoing a 7-day security maintenance. Your funds are safe, and access will resume soon." |
| **302** | `Auditor_Not_Authorized` | Only technical auditors can execute security vetoes. | "This action requires technical auditor permissions." |
| **303** | `Auditor_Already_Signed` | An auditor cannot sign the same stop request twice. | "You have already signed this security action." |
| **401** | `Dead_Contract_Only` | Cleanup can only be performed on fully completed streams. | "This contract is still active. Finalization can only occur after 100% completion." |
| **402** | `Insufficient_Platform_Fee` | Not enough fees collected to pay the cleanup bounty. | "Social cleanup bounty is currently unavailable for this contract." |
| **501** | `Only_Owner_Can_Update_XP` | Only the student who owns the profile NFT can update their experience points. | "You must be the owner of this profile to gain XP from this activity." |
| **502** | `Only_Owner_Can_Add_Achievements` | Achievements can only be unlocked by the profile owner. | "You must be the owner of this profile to unlock achievements." |
| **503** | `Transfer_Not_Authorized` | You do not have permission to transfer this profile NFT. | "You are not authorized to transfer this profile." |

---

> [!TIP]
> If you encounter an error not listed here, please check our Discord support channel with your transaction hash.
