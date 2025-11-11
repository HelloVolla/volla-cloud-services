//[client](../../../index.md)/[org.holochain.androidserviceruntime.client](../index.md)/[AppStatusFfi](index.md)

# AppStatusFfi

sealed class [AppStatusFfi](index.md)

#### Inheritors

| |
|---|
| [Disabled](-disabled/index.md) |
| [Enabled](-enabled/index.md) |
| [AwaitingMemproofs](-awaiting-memproofs/index.md) |

## Types

| Name | Summary |
|---|---|
| [AwaitingMemproofs](-awaiting-memproofs/index.md) | [androidJvm]<br>object [AwaitingMemproofs](-awaiting-memproofs/index.md) : [AppStatusFfi](index.md) |
| [Companion](-companion/index.md) | [androidJvm]<br>object [Companion](-companion/index.md) |
| [Disabled](-disabled/index.md) | [androidJvm]<br>data class [Disabled](-disabled/index.md)(val reason: [DisabledAppReasonFfi](../-disabled-app-reason-ffi/index.md)) : [AppStatusFfi](index.md) |
| [Enabled](-enabled/index.md) | [androidJvm]<br>object [Enabled](-enabled/index.md) : [AppStatusFfi](index.md) |

## Functions

| Name | Summary |
|---|---|
| [write](../-app-status-ffi-parceler/write.md) | [androidJvm]<br>open override fun [AppStatusFfi](index.md).[write](../-app-status-ffi-parceler/write.md)(parcel: [Parcel](https://developer.android.com/reference/kotlin/android/os/Parcel.html), flags: [Int](https://kotlinlang.org/api/core/kotlin-stdlib/kotlin/-int/index.html)) |