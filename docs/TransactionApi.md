# \TransactionApi

All URIs are relative to *https://mainnet.zklighter.elliot.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**account_txs**](TransactionApi.md#account_txs) | **GET** /api/v1/accountTxs | accountTxs
[**block_txs**](TransactionApi.md#block_txs) | **GET** /api/v1/blockTxs | blockTxs
[**deposit_history**](TransactionApi.md#deposit_history) | **GET** /api/v1/deposit/history | deposit_history
[**next_nonce**](TransactionApi.md#next_nonce) | **GET** /api/v1/nextNonce | nextNonce
[**send_tx**](TransactionApi.md#send_tx) | **POST** /api/v1/sendTx | sendTx
[**send_tx_batch**](TransactionApi.md#send_tx_batch) | **POST** /api/v1/sendTxBatch | sendTxBatch
[**transfer_history**](TransactionApi.md#transfer_history) | **GET** /api/v1/transfer/history | transfer_history
[**tx**](TransactionApi.md#tx) | **GET** /api/v1/tx | tx
[**tx_from_l1_tx_hash**](TransactionApi.md#tx_from_l1_tx_hash) | **GET** /api/v1/txFromL1TxHash | txFromL1TxHash
[**txs**](TransactionApi.md#txs) | **GET** /api/v1/txs | txs
[**withdraw_history**](TransactionApi.md#withdraw_history) | **GET** /api/v1/withdraw/history | withdraw_history



## account_txs

> models::Txs account_txs(limit, by, value, authorization, index, types, auth)
accountTxs

Get transactions of a specific account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | **i64** |  | [required] |
**by** | **String** |  | [required] |
**value** | **String** |  | [required] |
**authorization** | Option<**String**> |  |  |
**index** | Option<**i64**> |  |  |
**types** | Option<[**Vec<i32>**](i32.md)> |  |  |
**auth** | Option<**String**> |  |  |

### Return type

[**models::Txs**](Txs.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## block_txs

> models::Txs block_txs(by, value)
blockTxs

Get transactions in a block

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**by** | **String** |  | [required] |
**value** | **String** |  | [required] |

### Return type

[**models::Txs**](Txs.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## deposit_history

> models::DepositHistory deposit_history(account_index, l1_address, authorization, auth, cursor, filter)
deposit_history

Get deposit history

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**l1_address** | **String** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |
**cursor** | Option<**String**> |  |  |
**filter** | Option<**String**> |  |  |

### Return type

[**models::DepositHistory**](DepositHistory.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## next_nonce

> models::NextNonce next_nonce(account_index, api_key_index)
nextNonce

Get next nonce for a specific account and api key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**api_key_index** | **i32** |  | [required] |

### Return type

[**models::NextNonce**](NextNonce.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## send_tx

> models::RespSendTx send_tx(tx_type, tx_info, price_protection)
sendTx

You need to sign the transaction body before sending it to the server. More details can be found in the Get Started docs: [Get Started For Programmers](https://apidocs.lighter.xyz/docs/get-started-for-programmers)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tx_type** | **i32** |  | [required] |
**tx_info** | **String** |  | [required] |
**price_protection** | Option<**bool**> |  |  |[default to true]

### Return type

[**models::RespSendTx**](RespSendTx.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## send_tx_batch

> models::RespSendTxBatch send_tx_batch(tx_types, tx_infos)
sendTxBatch

You need to sign the transaction body before sending it to the server. More details can be found in the Get Started docs: [Get Started For Programmers](https://apidocs.lighter.xyz/docs/get-started-for-programmers)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tx_types** | **String** |  | [required] |
**tx_infos** | **String** |  | [required] |

### Return type

[**models::RespSendTxBatch**](RespSendTxBatch.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## transfer_history

> models::TransferHistory transfer_history(account_index, authorization, auth, cursor)
transfer_history

Get transfer history

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |
**cursor** | Option<**String**> |  |  |

### Return type

[**models::TransferHistory**](TransferHistory.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tx

> models::EnrichedTx tx(by, value)
tx

Get transaction by hash or sequence index

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**by** | **String** |  | [required] |
**value** | **String** |  | [required] |

### Return type

[**models::EnrichedTx**](EnrichedTx.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## tx_from_l1_tx_hash

> models::EnrichedTx tx_from_l1_tx_hash(hash)
txFromL1TxHash

Get L1 transaction by L1 transaction hash

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hash** | **String** |  | [required] |

### Return type

[**models::EnrichedTx**](EnrichedTx.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## txs

> models::Txs txs(limit, index)
txs

Get transactions which are already packed into blocks

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | **i64** |  | [required] |
**index** | Option<**i64**> |  |  |

### Return type

[**models::Txs**](Txs.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## withdraw_history

> models::WithdrawHistory withdraw_history(account_index, authorization, auth, cursor, filter)
withdraw_history

Get withdraw history

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |
**cursor** | Option<**String**> |  |  |
**filter** | Option<**String**> |  |  |

### Return type

[**models::WithdrawHistory**](WithdrawHistory.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

