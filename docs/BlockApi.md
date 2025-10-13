# \BlockApi

All URIs are relative to *https://mainnet.zklighter.elliot.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**block**](BlockApi.md#block) | **GET** /api/v1/block | block
[**blocks**](BlockApi.md#blocks) | **GET** /api/v1/blocks | blocks
[**current_height**](BlockApi.md#current_height) | **GET** /api/v1/currentHeight | currentHeight



## block

> models::Blocks block(by, value)
block

Get block by its height or commitment

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**by** | **String** |  | [required] |
**value** | **String** |  | [required] |

### Return type

[**models::Blocks**](Blocks.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## blocks

> models::Blocks blocks(limit, index, sort)
blocks

Get blocks

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | **i64** |  | [required] |
**index** | Option<**i64**> |  |  |
**sort** | Option<**String**> |  |  |[default to asc]

### Return type

[**models::Blocks**](Blocks.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## current_height

> models::CurrentHeight current_height()
currentHeight

Get current height

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::CurrentHeight**](CurrentHeight.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

