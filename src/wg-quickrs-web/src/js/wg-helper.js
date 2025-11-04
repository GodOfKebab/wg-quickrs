'use strict';

import FastEqual from "fast-deep-equal";

export default class WireGuardHelper {

    static validateField(fieldName, validator, originalValue, island_change_sum, field_color_lookup, ...validatorArgs) {
        const result = validator(...validatorArgs);

        if (result.error) {
            island_change_sum.errors[fieldName] = result.error;
            return [field_color_lookup["error"], island_change_sum];
        }

        if (!FastEqual(result.value, originalValue)) {
            island_change_sum.changed_fields[fieldName] = result.value;
            return [field_color_lookup["changed"], island_change_sum];
        }

        island_change_sum.changed_fields[fieldName] = null;
        island_change_sum.errors[fieldName] = null;
        return [field_color_lookup["unchanged"], island_change_sum];
    }

    static get_field_colors(is_new) {
        return {
            unchanged: is_new ? 'enabled:bg-green-200' : 'bg-white',
            changed: is_new ? 'enabled:bg-green-200' : 'enabled:bg-blue-200',
            error: 'enabled:bg-red-200',
        }
    }

    static get_div_colors(is_new) {
        return {
            unchanged: is_new ? 'bg-green-100' : 'bg-green-50',
            changed: is_new ? 'bg-green-100' : 'bg-blue-50',
            error: 'bg-red-50',
        }
    }

}
