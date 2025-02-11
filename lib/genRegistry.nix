{lib, ...}: pkgs: let
  inherit (builtins) deepSeq filter listToAttrs map parseDrvName seq tryEval;
  inherit (lib) filterAttrs flatten foldl isDerivation mapAttrsToList optional optionals removePrefix traceVal;

  registerPackage = name: value: let
    safeValue = tryEval value;
    safeVal = safeValue.value;

    safeRegistryValue = tryEval (deepSeq registryValue registryValue);
    registryValue = {
      pname = safeVal.pname or (parseDrvName safeVal.name).name or null;
      version = safeVal.version or null;
      storePaths = let
        getOutput = out: {
          name = out;
          value = let
            outPath = tryEval safeVal.${out}.outPath;
          in
            if outPath.success
            then removePrefix "/nix/store/" outPath.value
            else "<broken>";
        };

        outputs-list = map getOutput (safeVal.outputs or []);
      in
        listToAttrs outputs-list;

      propagatedBuildInputs = let
        collectInputs = inputList: let
          directInputs = lib.filter (x: x != null) (lib.lists.flatten inputList);
          recursiveInputs = lib.flatten (
            map
            (
              x:
                if lib.isAttrs x && x ? propagatedBuildInputs
                then collectInputs x.propagatedBuildInputs
                else []
            )
            directInputs
          );
        in
          lib.unique (directInputs ++ recursiveInputs);
      in
        map (x:
          removePrefix "/nix/store/" (
            if lib.isAttrs x
            then x.outPath
            else x
          )) (
          collectInputs (safeVal.propagatedBuildInputs or [])
        );

      propagatedNativeBuildInputs = let
        collectInputs = inputList: let
          directInputs = lib.filter (x: x != null) (lib.lists.flatten inputList);
          recursiveInputs = lib.flatten (
            map
            (
              x:
                if lib.isAttrs x && x ? propagatedNativeBuildInputs
                then collectInputs x.propagatedNativeBuildInputs
                else []
            )
            directInputs
          );
        in
          lib.unique (directInputs ++ recursiveInputs);
      in
        map (x:
          removePrefix "/nix/store/" (
            if lib.isAttrs x
            then x.outPath
            else x
          )) (
          collectInputs (safeVal.propagatedNativeBuildInputs or [])
        );

      meta = {
        description = safeVal.meta.description or null;
        homepage = safeVal.meta.homepage or null;
        license = safeVal.meta.license or null;
        longDescription = safeVal.meta.longDescription or null;
      };
    };

    platformForAvailability = {system = pkgs.system or builtins.currentSystem;};
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
      map
      (item: {
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
  registry
