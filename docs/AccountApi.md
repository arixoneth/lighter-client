# \AccountApi

All URIs are relative to *https://mainnet.zklighter.elliot.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**account**](AccountApi.md#account) | **GET** /api/v1/account | account
[**account_limits**](AccountApi.md#account_limits) | **GET** /api/v1/accountLimits | accountLimits
[**account_metadata**](AccountApi.md#account_metadata) | **GET** /api/v1/accountMetadata | accountMetadata
[**accounts_by_l1_address**](AccountApi.md#accounts_by_l1_address) | **GET** /api/v1/accountsByL1Address | accountsByL1Address
[**apikeys**](AccountApi.md#apikeys) | **GET** /api/v1/apikeys | apikeys
[**change_account_tier**](AccountApi.md#change_account_tier) | **POST** /api/v1/changeAccountTier | changeAccountTier
[**l1_metadata**](AccountApi.md#l1_metadata) | **GET** /api/v1/l1Metadata | l1Metadata
[**liquidations**](AccountApi.md#liquidations) | **GET** /api/v1/liquidations | liquidations
[**pnl**](AccountApi.md#pnl) | **GET** /api/v1/pnl | pnl
[**position_funding**](AccountApi.md#position_funding) | **GET** /api/v1/positionFunding | positionFunding
[**public_pools**](AccountApi.md#public_pools) | **GET** /api/v1/publicPools | publicPools
[**public_pools_metadata**](AccountApi.md#public_pools_metadata) | **GET** /api/v1/publicPoolsMetadata | publicPoolsMetadata



## account

> models::DetailedAccounts account(by, value)
account

Get account by account's index. <br>More details about account index: [Account Index](https://apidocs.lighter.xyz/docs/account-index)<hr>**Response Description:**<br><br>1) **Status:** 1 is active 0 is inactive.<br>2) **Collateral:** The amount of collateral in the account.<hr>**Position Details Description:**<br>1) **OOC:** Open order count in that market.<br>2) **Sign:** 1 for Long, -1 for Short.<br>3) **Position:** The amount of position in that market.<br>4) **Avg Entry Price:** The average entry price of the position.<br>5) **Position Value:** The value of the position.<br>6) **Unrealized PnL:** The unrealized profit and loss of the position.<br>7) **Realized PnL:** The realized profit and loss of the position.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**by** | **String** |  | [required] |
**value** | **String** |  | [required] |

### Return type

[**models::DetailedAccounts**](DetailedAccounts.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## account_limits

> models::AccountLimits account_limits(account_index, authorization, auth)
accountLimits

Get account limits

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |

### Return type

[**models::AccountLimits**](AccountLimits.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## account_metadata

> models::AccountMetadatas account_metadata(by, value, authorization, auth)
accountMetadata

Get account metadatas

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**by** | **String** |  | [required] |
**value** | **String** |  | [required] |
**authorization** | Option<**String**> |  |  |
**auth** | Option<**String**> |  |  |

### Return type

[**models::AccountMetadatas**](AccountMetadatas.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## accounts_by_l1_address

> models::SubAccounts accounts_by_l1_address(l1_address)
accountsByL1Address

Get accounts by l1_address returns all accounts associated with the given L1 address

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**l1_address** | **String** |  | [required] |

### Return type

[**models::SubAccounts**](SubAccounts.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## apikeys

> models::AccountApiKeys apikeys(account_index, api_key_index)
apikeys

Get account api key. Set `api_key_index` to 255 to retrieve all api keys associated with the account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**api_key_index** | Option<**i32**> |  |  |[default to 255]

### Return type

[**models::AccountApiKeys**](AccountApiKeys.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## change_account_tier

> models::RespChangeAccountTier change_account_tier(account_index, new_tier, authorization, auth)
changeAccountTier

Change account tier

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**new_tier** | **String** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |

### Return type

[**models::RespChangeAccountTier**](RespChangeAccountTier.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## l1_metadata

> models::L1Metadata l1_metadata(l1_address, authorization, auth)
l1Metadata

Get L1 metadata

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**l1_address** | **String** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |

### Return type

[**models::L1Metadata**](L1Metadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## liquidations

> models::LiquidationInfos liquidations(account_index, limit, authorization, auth, market_id, cursor)
liquidations

Get liquidation infos

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**limit** | **i64** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |
**market_id** | Option<**i32**> |  |  |[default to 255]
**cursor** | Option<**String**> |  |  |

### Return type

[**models::LiquidationInfos**](LiquidationInfos.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pnl

> models::AccountPnL pnl(by, value, resolution, start_timestamp, end_timestamp, count_back, authorization, auth, ignore_transfers)
pnl

Get account PnL chart

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**by** | **String** |  | [required] |
**value** | **String** |  | [required] |
**resolution** | **String** |  | [required] |
**start_timestamp** | **i64** |  | [required] |
**end_timestamp** | **i64** |  | [required] |
**count_back** | **i64** |  | [required] |
**authorization** | Option<**String**> |  |  |
**auth** | Option<**String**> |  |  |
**ignore_transfers** | Option<**bool**> |  |  |[default to false]

### Return type

[**models::AccountPnL**](AccountPnL.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## position_funding

> models::PositionFundings position_funding(account_index, limit, authorization, auth, market_id, cursor, side)
positionFunding

Get accounts position fundings

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**limit** | **i64** |  | [required] |
**authorization** | Option<**String**> |  |  |
**auth** | Option<**String**> |  |  |
**market_id** | Option<**i32**> |  |  |[default to 255]
**cursor** | Option<**String**> |  |  |
**side** | Option<**String**> |  |  |[default to all]

### Return type

[**models::PositionFundings**](PositionFundings.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## public_pools

> models::PublicPools public_pools(index, limit, authorization, auth, filter, account_index)
publicPools

Get public pools

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**index** | **i64** |  | [required] |
**limit** | **i64** |  | [required] |
**authorization** | Option<**String**> |  |  |
**auth** | Option<**String**> |  |  |
**filter** | Option<**String**> |  |  |
**account_index** | Option<**i64**> |  |  |

### Return type

[**models::PublicPools**](PublicPools.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## public_pools_metadata

> models::RespPublicPoolsMetadata public_pools_metadata(index, limit, authorization, auth, filter, account_index)
publicPoolsMetadata

Get public pools metadata

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**index** | **i64** |  | [required] |
**limit** | **i64** |  | [required] |
**authorization** | Option<**String**> |  |  |
**auth** | Option<**String**> |  |  |
**filter** | Option<**String**> |  |  |
**account_index** | Option<**i64**> |  |  |

### Return type

[**models::RespPublicPoolsMetadata**](RespPublicPoolsMetadata.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

