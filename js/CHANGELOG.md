# CHANGELOG

## 0.4.1

### Compatibility
- Sails-RS v0.8.0
- Gear v1.8.0

### Changes
- Add explicit export of `IMethodReturnType` from `TransactionBuilder`

## 0.4.0

### Compatibility
- Sails-RS v0.8.0
- Gear v1.8.0

### Changes
- Support ReplyCode from `@gear-js/api` in https://github.com/gear-tech/sails/pull/893
From now on, `response` function will throw an error if the program's reply is successful.
- Update types in ctor generation (https://github.com/gear-tech/sails/issues/786)
- Unpin `sails-js` peer dependencies

## 0.3.2

### Compatibility
- Sails-RS v0.7.1

### Changes
- Ability to get `gasInfo` in `TransactionBuilder` in https://github.com/gear-tech/sails/pull/745

## 0.3.1

### Compatibility
- Sails-RS v0.7.0

### Changes
- Setup automated releases in https://github.com/gear-tech/sails/pull/608
- Update dependencies in https://github.com/gear-tech/sails/pull/709
