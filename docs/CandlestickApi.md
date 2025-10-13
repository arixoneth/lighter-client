# \CandlestickApi

All URIs are relative to *https://mainnet.zklighter.elliot.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**candlesticks**](CandlestickApi.md#candlesticks) | **GET** /api/v1/candlesticks | candlesticks
[**fundings**](CandlestickApi.md#fundings) | **GET** /api/v1/fundings | fundings



## candlesticks

> models::Candlesticks candlesticks(market_id, resolution, start_timestamp, end_timestamp, count_back, set_timestamp_to_end)
candlesticks

Get candlesticks

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**market_id** | **i32** |  | [required] |
**resolution** | **String** |  | [required] |
**start_timestamp** | **i64** |  | [required] |
**end_timestamp** | **i64** |  | [required] |
**count_back** | **i64** |  | [required] |
**set_timestamp_to_end** | Option<**bool**> |  |  |[default to false]

### Return type

[**models::Candlesticks**](Candlesticks.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fundings

> models::Fundings fundings(market_id, resolution, start_timestamp, end_timestamp, count_back)
fundings

Get fundings

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**market_id** | **i32** |  | [required] |
**resolution** | **String** |  | [required] |
**start_timestamp** | **i64** |  | [required] |
**end_timestamp** | **i64** |  | [required] |
**count_back** | **i64** |  | [required] |

### Return type

[**models::Fundings**](Fundings.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

