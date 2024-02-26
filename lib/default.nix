{lib, ...}: let
in {
  genRegistry = pkgs: let
    inherit (builtins) currentSystem deepSeq listToAttrs map parseDrvName seq tryEval;
    inherit (lib) filterAttrs flatten foldl isDerivation mapAttrsToList optional optionals traceVal;

    registerPackage = name: value: let
      safeValue = tryEval value;

      safeRegistryValue = tryEval (deepSeq registryValue registryValue);
      registryValue = {
        pname = value.pname or (parseDrvName value.name).name or null;
        version = value.version or null;

        meta = {
          description = value.meta.description or null;
          homepage = value.meta.homepage or null;
          license = value.meta.license or null;
          longDescription = value.meta.longDescription or null;
        };
      };

      platformForAvailability = {system = pkgs.system or currentSystem;};
      isAvailableOn = tryEval (lib.meta.availableOn platformForAvailability safeValue.value);
      available = safeValue.success && isDerivation value && isAvailableOn.success && isAvailableOn.value;

      checkRegistryCondition = prev: {
        reason,
        ok,
      }: let
        isOk =
          if !ok
          then seq (traceVal "${name}: ${reason}") false
          else true;
      in
        # change to `prev && isOk` to debug why a value isn't included
        prev && ok;

      shouldBeInRegistry = foldl checkRegistryCondition true [
        {
          reason = "not available on ${platformForAvailability.system}";
          ok = available;
        }
        {
          reason = "failed eval";
          ok = safeRegistryValue.success;
        }
        {
          reason = "no pname";
          ok = safeRegistryValue.value.pname != null;
        }
      ];
    in
      optional shouldBeInRegistry {
        inherit name;
        value = let
          filtered-toplevel-attrs = filterAttrs (_: v: v != null) safeRegistryValue.value;
          filtered-meta-attrs = filterAttrs (_: v: v != null) safeRegistryValue.value.meta;
        in
          filtered-toplevel-attrs // {meta = filtered-meta-attrs;};
      };

    registerScope = scope-name: scope: let
      safeScope = tryEval scope;

      list-of-scope-packages = mapAttrsToList registerPackage safeScope.value;
      scope-registry-inner = flatten list-of-scope-packages;
      scope-registry =
        map (item: {
          name = "${scope-name}.${item.name}";
          value = item.value;
        })
        scope-registry-inner;

      shouldBeInRegistry = safeScope.success && safeScope.value ? recurseForDerivations && safeScope.value.recurseForDerivations;
    in
      optionals shouldBeInRegistry scope-registry;

    list-of-registry-packages = mapAttrsToList registerPackage pkgs;
    registry-items = flatten list-of-registry-packages;

    scoped-registries = flatten (mapAttrsToList registerScope pkgs);
    registry = listToAttrs (registry-items ++ scoped-registries);
  in
    registry;
}
