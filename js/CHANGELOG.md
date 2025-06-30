# CHANGELOG

## master
### Changes
- Skip type file generation if types are not present in the IDL
- Improve encoding / decoding in the `TransactionBuilder` class
- Capitalize enum variant names in generated files

## 0.4.2
### Compatibility
- Sails-RS v0.8.1
- Gear v1.8.1

### Changes
- Bump `@gear-js/api` to `v0.42.0` in https://github.com/gear-tech/sails/pull/933
- Support `Program` class from `@gear-js/api` to keep track on program upgrades in https://github.com/gear-tech/sails/pull/933

## 0.4.1

### Compatibility
- Sails-RS v0.8.0
- Gear v1.8.0

### Changes
- Add explicit export of `IMethodReturnType` from `TransactionBuilder`

---

## 0.4.0

### Compatibility
- Sails-RS v0.8.0
- Gear v1.8.0

### Changes
- Support ReplyCode from `@gear-js/api` in https://github.com/gear-tech/sails/pull/893
From now on, `response` function will throw an error if the program's reply is successful.
- Update types in ctor generation (https://github.com/gear-tech/sails/issues/786)
- Unpin `sails-js` peer dependencies

---

## 0.3.2

### Compatibility
- Sails-RS v0.7.1

### Changes
- Ability to get `gasInfo` in `TransactionBuilder` in https://github.com/gear-tech/sails/pull/745

---

## 0.3.1

### Compatibility
- Sails-RS v0.7.0

### Changes
- Setup automated releases in https://github.com/gear-tech/sails/pull/608
- Update dependencies in https://github.com/gear-tech/sails/pull/709
