require("balatro-vs_globals")
require("balatro-vs_network")
InteractionContext = require("balatro-vs_interaction").InteractionContext
Interaction = require("balatro-vs_interaction").Interaction

--Timer related
local EventTimer = require("balatro-vs_utils").EventTimer

local timer_callback = {
    on_wait_for_user_action_over = function()
        if G.STATE == G.STATES.SELECTING_HAND then --Force empty sent if timer runs out
            bvs_debug("Timer ran out, sending a random card")

            if G.OVERLAY_MENU then --On the run infos overlay, exit it
                G.FUNCS.exit_overlay_menu()
            end

            local highlighted_cards = G.hand:get_highlighted_cards_index()
            if #highlighted_cards == 0 then
                --Highlight card if none are highlighted
                local index = math.random(1, #G.hand.cards)

                local card = G.hand.cards[index]
                if card then
                    card:click()
                end
            end

            G.FUNCS.lock_highlighted_card()
        end
    end,
    on_shop_action_over = function()
        if G.OVERLAY_MENU then --On the run infos overlay, exit it
            G.FUNCS.exit_overlay_menu()
        end
        if lume.any({
                    G.STATES.STANDARD_PACK,
                    G.STATES.TAROT_PACK,
                    G.STATES.PLANET_PACK,
                    G.STATES.SPECTRAL_PACK,
                    G.STATES.BUFFOON_PACK },
                function(states) return G.STATE == states end)
        then
            --Timer runs out on pack selection, skip it first

            G.FUNCS.skip_booster()
        end

        G.FUNCS.network_wait_for_opponent_action_on_end_shop()
    end,
    on_cashout_action_over = function()
        if G.STATE == G.STATES.ROUND_EVAL then --No waiting indefinitely on round eval
            if G.OVERLAY_MENU then             --On the run infos overlay, exit it
                G.FUNCS.exit_overlay_menu()
            end

            G.FUNCS.cash_out({
                config = {
                    button = nil
                }
            })
        end
    end,
}

function toggle_click(cards, can)
    lume.each(cards, function(card)
        card.states.click.can = can
        card.states.drag.can = can
        if not can then
            card.highlighted = false
        end
    end)
end

function on_create_timer(duration, callback_name, should_start)
    local callback = timer_callback[callback_name]

    if not callback then
        bvs_debug("Callback not found")
        return
    end

    BALATRO_VS_CTX.interaction_context:handle_interaction({ on_new_timer = true }, {})

    BALATRO_VS_CTX.timer = EventTimer:new(duration, callback)

    BALATRO_VS_CTX.interaction_context:handle_interaction({ on_create_timer = true }, {})

    if should_start then
        if G.jokers then
            toggle_click(G.jokers.cards, true)
        end

        if G.consumeables then
            toggle_click(G.consumeables.cards, true)
        end

        BALATRO_VS_CTX.timer:start()
    end
end

function on_start_timer()
    if not BALATRO_VS_CTX.timer then
        bvs_debug("Timer is not created")
        return
    end

    if BALATRO_VS_CTX.timer.active then
        bvs_debug("Timer is already running")
        return
    end

    if G.jokers then
        toggle_click(G.jokers.cards, true)
    end

    if G.consumeables then
        toggle_click(G.consumeables.cards, true)
    end

    BALATRO_VS_CTX.timer:start()
end

function on_pause_timer()
    if not BALATRO_VS_CTX.timer then
        bvs_debug("Timer is not created")
        return
    end

    if not BALATRO_VS_CTX.timer.active then
        bvs_debug("Timer is not running")
        return
    end

    BALATRO_VS_CTX.timer:pause()
end

function on_resume_timer()
    if not BALATRO_VS_CTX.timer then
        bvs_debug("Timer is not created")
        return
    end

    if not BALATRO_VS_CTX.timer.active then
        bvs_debug("Timer is not running")
        return
    end

    BALATRO_VS_CTX.timer:resume()
end

function on_stop_timer()
    if not BALATRO_VS_CTX.timer then
        bvs_debug("Timer is not created")
        return
    end

    if not BALATRO_VS_CTX.timer.active then
        bvs_debug("Timer is not running")
        return
    end

    BALATRO_VS_CTX.timer:stop()
    BALATRO_VS_CTX.timer = nil

    --prevent interaction until next timer
    for _, jokers in ipairs(G.opponent_jokers) do
        if jokers.highlighted then
            jokers.highlighted = false
        end
    end

    for _, consumeables in ipairs(G.consumeables) do
        if consumeables.highlighted then
            consumeables.highlighted = false
        end
    end

    if G.jokers then
        toggle_click(G.jokers.cards, false)
    end

    if G.consumeables then
        toggle_click(G.consumeables.cards, false)
    end
end

--Alignement related
local function check_alignment(table1, table2)
    if table1 == nil or table2 == nil then return false end
    if #table1 ~= #table2 then return false end
    for i, unique_val in ipairs(table1) do
        if unique_val ~= table2[i] then
            return false
        end
    end
    return true
end

local function find_index_changes(original, new, dragged_card)
    local index_changes = {}

    for new_index, value in ipairs(new) do
        if original[new_index] == value then
            index_changes[new_index] = 'SAME'
        else
            index_changes[new_index] = 'MOVED'
        end
    end

    local moved = {}
    for index, _ in ipairs(index_changes) do
        if index_changes[index] == 'MOVED' then
            table.insert(moved, index)
        end
    end

    local should_reverse = false
    for ind, value in ipairs(moved) do
        if value == dragged_card then
            if ind == 1 then
                should_reverse = true
            end
            break
        end
    end

    if should_reverse then
        local reversed = {}
        for i = #moved, 1, -1 do
            table.insert(reversed, moved[i])
        end
        moved = reversed
    end

    return moved
end

local function update_alignment(type, items, dragged_card)
    local new_alignment = lume.map(items, function(item) return item.unique_val end)

    if BALATRO_VS_CTX.alignements[type] == nil or #new_alignment ~= #BALATRO_VS_CTX.alignements[type] then
        BALATRO_VS_CTX.alignements[type] = copy_table(new_alignment)
    else
        if not check_alignment(BALATRO_VS_CTX.alignements[type], new_alignment) and #new_alignment == #BALATRO_VS_CTX.alignements[type] then
            if dragged_card ~= -1 then
                local index_changes = find_index_changes(BALATRO_VS_CTX.alignements[type], new_alignment, dragged_card)
                G.FUNCS.network_send_to_opponent_new_cards_alignement(index_changes, type)
            end

            BALATRO_VS_CTX.alignements[type] = copy_table(new_alignment)
        end
    end
end

function update_hand_cards_alignment(cards, dragged_card)
    update_alignment('hand', cards, dragged_card)
end

function update_jokers_alignment(jokers, dragged_card)
    update_alignment('jokers', jokers, dragged_card)
end

function update_consumeables_alignment(consumeables, dragged_card)
    update_alignment('consumeables', consumeables, dragged_card)
end

function backup_progress()
    BALATRO_VS_CTX.progress = {
        all_unlocked = G.PROFILES[G.SETTINGS.profile].all_unlocked,
        centers = lume.serialize(G.P_CENTERS),
        blinds = lume.serialize(G.P_BLINDS),
        tags = lume.serialize(G.P_TAGS),
    }
end

BALATRO_VS_CTX = {
    network = {
        has_confirmed_matchmaking = false,
        current_friendly_room_code = "",
        is_updating = false,
        is_error = false,
        is_live = false,
        rtt = "0 ms",
        should_reset = false,
        has_opponent_maestro_and_highlighted_cards = false,
    },
    is_in_shop_and_ready = false,
    alignements = {
        hand = {},
        jokers = {},
        consumeables = {}
    },
    main_goal = 10,
    timer = nil,
    is_opponent_first_reroll_shop = true,
    rounds_played = 1, --TODO: Reset on Campfire sell
    interaction_context = InteractionContext:new(),
    progress = {

    }
}
