# \OrderApi

All URIs are relative to *https://mainnet.zklighter.elliot.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**account_active_orders**](OrderApi.md#account_active_orders) | **GET** /api/v1/accountActiveOrders | accountActiveOrders
[**account_inactive_orders**](OrderApi.md#account_inactive_orders) | **GET** /api/v1/accountInactiveOrders | accountInactiveOrders
[**exchange_stats**](OrderApi.md#exchange_stats) | **GET** /api/v1/exchangeStats | exchangeStats
[**export**](OrderApi.md#export) | **GET** /api/v1/export | export
[**order_book_details**](OrderApi.md#order_book_details) | **GET** /api/v1/orderBookDetails | orderBookDetails
[**order_book_orders**](OrderApi.md#order_book_orders) | **GET** /api/v1/orderBookOrders | orderBookOrders
[**order_books**](OrderApi.md#order_books) | **GET** /api/v1/orderBooks | orderBooks
[**recent_trades**](OrderApi.md#recent_trades) | **GET** /api/v1/recentTrades | recentTrades
[**trades**](OrderApi.md#trades) | **GET** /api/v1/trades | trades



## account_active_orders

> models::Orders account_active_orders(account_index, market_id, authorization, auth)
accountActiveOrders

Get account active orders. `auth` can be generated using the SDK.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**market_id** | **i32** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |

### Return type

[**models::Orders**](Orders.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## account_inactive_orders

> models::Orders account_inactive_orders(account_index, limit, authorization, auth, market_id, ask_filter, between_timestamps, cursor)
accountInactiveOrders

Get account inactive orders

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**limit** | **i64** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |
**market_id** | Option<**i32**> |  |  |[default to 255]
**ask_filter** | Option<**i32**> |  |  |[default to -1]
**between_timestamps** | Option<**String**> |  |  |
**cursor** | Option<**String**> |  |  |

### Return type

[**models::Orders**](Orders.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## exchange_stats

> models::ExchangeStats exchange_stats()
exchangeStats

Get exchange stats

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ExchangeStats**](ExchangeStats.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## export

> models::ExportData export(r#type, authorization, auth, account_index, market_id)
export

Export data

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**r#type** | **String** |  | [required] |
**authorization** | Option<**String**> |  |  |
**auth** | Option<**String**> |  |  |
**account_index** | Option<**i64**> |  |  |[default to -1]
**market_id** | Option<**i32**> |  |  |[default to 255]

### Return type

[**models::ExportData**](ExportData.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## order_book_details

> models::OrderBookDetails order_book_details(market_id)
orderBookDetails

Get order books metadata

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**market_id** | Option<**i32**> |  |  |[default to 255]

### Return type

[**models::OrderBookDetails**](OrderBookDetails.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## order_book_orders

> models::OrderBookOrders order_book_orders(market_id, limit)
orderBookOrders

Get order book orders

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**market_id** | **i32** |  | [required] |
**limit** | **i64** |  | [required] |

### Return type

[**models::OrderBookOrders**](OrderBookOrders.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## order_books

> models::OrderBooks order_books(market_id)
orderBooks

Get order books metadata.<hr>**Response Description:**<br><br>1) **Taker and maker fees** are in percentage.<br>2) **Min base amount:** The amount of base token that can be traded in a single order.<br>3) **Min quote amount:** The amount of quote token that can be traded in a single order.<br>4) **Supported size decimals:** The number of decimal places that can be used for the size of the order.<br>5) **Supported price decimals:** The number of decimal places that can be used for the price of the order.<br>6) **Supported quote decimals:** Size Decimals + Quote Decimals.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**market_id** | Option<**i32**> |  |  |[default to 255]

### Return type

[**models::OrderBooks**](OrderBooks.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## recent_trades

> models::Trades recent_trades(market_id, limit)
recentTrades

Get recent trades

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**market_id** | **i32** |  | [required] |
**limit** | **i64** |  | [required] |

### Return type

[**models::Trades**](Trades.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## trades

> models::Trades trades(sort_by, limit, authorization, auth, market_id, account_index, order_index, sort_dir, cursor, from, ask_filter)
trades

Get trades

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sort_by** | **String** |  | [required] |
**limit** | **i64** |  | [required] |
**authorization** | Option<**String**> |  |  |
**auth** | Option<**String**> |  |  |
**market_id** | Option<**i32**> |  |  |[default to 255]
**account_index** | Option<**i64**> |  |  |[default to -1]
**order_index** | Option<**i64**> |  |  |
**sort_dir** | Option<**String**> |  |  |[default to desc]
**cursor** | Option<**String**> |  |  |
**from** | Option<**i64**> |  |  |[default to -1]
**ask_filter** | Option<**i32**> |  |  |[default to -1]

### Return type

[**models::Trades**](Trades.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

