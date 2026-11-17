# src/api/client.test.ts

Tests for the command client: conversion of command errors and other values
into `ApiError`, message formatting, locked and wrong-password detection, and that `call()`
passes arguments and converts rejections.
