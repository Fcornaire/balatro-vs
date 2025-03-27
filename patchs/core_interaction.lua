--Index based random element
local function custom_pseudorandom_element(array, seed)
    math.randomseed(seed)

    local index = math.random(1, #array)

    return array[index]
end

local function custom_pseudoshuffle(array, seed)
    math.randomseed(seed)

    for i = #array, 2, -1 do
        local j = math.random(i)
        array[i], array[j] = array[j], array[i]
    end
end

local function custom_copy_table(tbl)
    local copy = {}
    for k, v in pairs(tbl) do
        if type(v) == "function" then
            copy[k] = nil
        elseif type(v) == "table" then
            copy[k] = custom_copy_table(v)
        elseif type(v) == "string" then
            copy[k] = v .. ""
        else
            copy[k] = v
        end
    end
    return copy
end


local function common_on_card_click(owner, interaction_context) -- Common function for on_card_click
    if next(interaction_context.target.children) == nil then
        for _, consumeables in pairs(G.consumeables.cards) do
            consumeables.highlighted = false
            if consumeables.ability and consumeables.ability.targets then
                consumeables.ability.targets = {}
            end
        end

        for _, joker in pairs(G.opponent_jokers.cards) do
            joker.highlighted = false
        end

        return
    end

    if owner ~= interaction_context.target then
        if interaction_context.target and getmetatable(interaction_context.target) == Card and interaction_context.target.area == G.opponent_jokers then
            if owner.highlighted then
                if interaction_context.target.highlighted then
                    owner.ability.targets = {}
                    play_sound('cardSlide2', nil, 0.3)
                else
                    for _, joker in pairs(G.opponent_jokers.cards) do
                        joker.highlighted = false
                    end

                    if owner.ability.targets and #owner.ability.targets > 0 then
                        owner.ability.targets[1].highlighted = false
                    end
                    owner.ability.targets = {}
                    table.insert(owner.ability.targets, interaction_context.target)
                    play_sound('cardSlide1')
                end
                interaction_context.target.highlighted = not interaction_context.target.highlighted
            else
                owner.ability.targets = {}
            end
        end
    else
        for _, joker in pairs(G.opponent_jokers.cards) do
            joker.highlighted = false
        end
    end
end

local function common_on_use(card, action) -- Common function for on_use
    if card.ability.targets and #card.ability.targets > 0 then
        action(card.ability.targets[1])

        card.ability.targets[1].highlighted = false
    end
end

function initialize_online_cards(game_object)
    game_object.ONLINE_CENTERS = {
        c_online_negate_joker_round = {
            key = 'c_online_negate_joker_round',
            order = 10,
            discovered = true,
            cost = 3,
            consumeable = true,
            name = "Negate",
            pos = { x = 5, y = 2 },
            set = "Spectral",
            config = { max_highlighted = 1 },
            description = {
                main = "Negate the (futures and on going) effect of a joker for this round",
                sub = "Click this card first, then the target joker and finally use this card"
            },
            can_use_consumeable = function(card)
                return card and card.ability.targets and #card.ability.targets > 0
            end,
            actions = {
                on_card_click = function(owner, interaction_context)
                    common_on_card_click(owner, interaction_context)
                end,
                on_use = function(card)
                    common_on_use(card, function(target)
                        target:set_debuff(true)
                        if is_online_card(target.ability.name) then
                            local center = get_online_center_config_by_name_or_key(target.ability.name)
                            if center and center.on_debuff then
                                center.on_debuff(target)
                            end
                        end
                    end)
                end
            },
            cleanups = {
                on_cashout = function(cards)
                    local card = cards and cards[1]
                    if card and card.debuff then
                        card:set_debuff(false)
                    end
                end
            }
        },
        j_online_begone_joker = {
            key = 'j_online_begone_joker',
            rarity = 4,
            order = 10,
            discovered = true,
            cost = 3,
            config = {},
            name = "Begone ! Joker",
            pos = { x = 9, y = 9 },
            set = "Joker",
            perishable_compat = true,
            description = {
                main = "Randomly destroy an opponent joker at the end of the round",
                sub = "The owner of this card has to pay {X} at the start of a round to keep this card or destroy it"
            },
            can_activate_more_than_once_per_chain = true,
            actions = {
                end_of_round_after_jokers = function(owner)
                    if owner.debuff then
                        return
                    end
                    if owner.params.is_opponent then
                        if #G.jokers.cards > 0 then
                            local to_remove = custom_pseudorandom_element(G.jokers.cards,
                                opponent_pseudoseed('begone_joker'))

                            if to_remove then
                                owner:juice_up()
                                play_sound('generic1')
                                to_remove:start_dissolve()
                            end
                        end
                    else
                        if #G.opponent_jokers.cards > 0 then
                            local to_remove = custom_pseudorandom_element(G.opponent_jokers.cards,
                                opponent_pseudoseed('begone_joker'))

                            if to_remove then
                                owner:juice_up()
                                play_sound('generic1')
                                to_remove:start_dissolve()
                            end
                        end
                    end
                end,
                first_hand_drawn_joker = function(owner)
                    if owner.debuff then
                        return
                    end

                    if owner.params.is_opponent then
                        if G.GAME.opponent_dollars >= owner.ability.begone_cost then
                            opponent_ease_dollars(-owner.ability.begone_cost, true)
                            owner:juice_up()
                            play_sound('card1')
                        else
                            owner:start_dissolve()
                        end
                    else
                        if G.GAME.dollars >= owner.ability.begone_cost then
                            ease_dollars(-owner.ability.begone_cost, true)
                            owner:juice_up()
                            play_sound('card1')
                        else
                            owner:start_dissolve()
                        end
                    end

                    owner.ability.begone_cost = owner.ability.begone_cost + 5
                end

            },
            cleanups = {

            }
        },
        j_online_jester = {
            key = 'j_online_jester',
            rarity = 4,
            order = 10,
            discovered = false,
            cost = 2,
            config = {},
            name = "Jester",
            pos = { x = 9, y = 9 },
            set = "Joker",
            perishable_compat = true,
            description = {
                main = "At the start of every round, randomly swap opponent jokers centers (restore if destroyed)",
                sub = "The original jokers effects remain the same"
            },
            can_activate_more_than_once_per_chain = true,
            on_debuff = function(card)
                local center = get_online_center_config_by_name_or_key(card.ability.name)
                if center and center.cleanups then
                    center.cleanups['instant'](card.params.is_opponent)
                end
            end,
            actions = {
                first_hand_drawn_joker = function(owner)
                    if owner.debuff then
                        return
                    end

                    owner:juice_up()
                    if owner.params.is_opponent then
                        for i, card in pairs(G.jokers.cards) do
                            if not card.config.old_center then
                                card.config.old_center = is_online_card(card.config.center.key) and
                                    lume.serialize(get_center_for_online_card_by_name(card.config.center.name))
                                    or (card.config.center and lume.serialize(card.config.center) or '')
                            end
                            local random_joker = custom_pseudorandom_element(G.jokers.cards,
                                opponent_pseudoseed('jester'))
                            card.config.center = random_joker.config.old_center and
                                lume.deserialize(random_joker.config.old_center) or random_joker.config.center
                            card:set_sprites(card.config.center)
                        end
                        custom_pseudoshuffle(G.jokers.cards, opponent_pseudoseed('jester'))
                    else
                        for i, card in pairs(G.opponent_jokers.cards) do
                            if not card.config.old_center then
                                card.config.old_center = is_online_card(card.config.center.key) and
                                    lume.serialize(get_center_for_online_card_by_name(card.config.center.name))
                                    or (card.config.center and lume.serialize(card.config.center) or '')
                            end
                            local random_joker = custom_pseudorandom_element(G.opponent_jokers.cards,
                                opponent_pseudoseed('jester'))
                            card.config.center = random_joker.config.old_center and
                                lume.deserialize(random_joker.config.old_center) or random_joker.config.center
                            card:set_sprites(card.config.center)
                        end
                        custom_pseudoshuffle(G.opponent_jokers.cards, opponent_pseudoseed('jester'))
                    end
                end
            },
            cleanups = {
                instant = function(is_opponent)
                    if is_opponent then
                        for i, card in pairs(G.jokers.cards) do
                            if card.config.old_center then
                                card.config.center = lume.deserialize(card.config.old_center)
                                card:set_sprites(card.config.center)
                                card.config.old_center = nil
                            end
                        end
                    else
                        for i, card in pairs(G.opponent_jokers.cards) do
                            if card.config.old_center then
                                card.config.center = lume.deserialize(card.config.old_center)
                                card:set_sprites(card.config.center)
                                card.config.old_center = nil
                            end
                        end
                    end
                end
            }
        },
        c_online_eternal = {
            key = 'c_online_eternal',
            order = 10,
            discovered = true,
            cost = 3,
            consumeable = true,
            name = "Angel touch!",
            pos = { x = 5, y = 2 },
            set = "Spectral",
            config = { max_highlighted = 1 },
            description = {
                main = "Turn a joker into an eternal card",
                sub = "Click this card first, then the target joker and finally use this card"
            },
            can_use_consumeable = function(card)
                return card and card.ability.targets and #card.ability.targets > 0
            end,
            actions = {
                on_card_click = function(owner, interaction_context)
                    common_on_card_click(owner, interaction_context)
                end,
                on_use = function(card)
                    common_on_use(card, function(target)
                        if target.config.center.eternal_compat and not target.ability.perishable then
                            target:set_eternal(true)
                        else
                            play_area_status_text("Not Allowed!")
                        end
                    end)
                end
            },
            cleanups = {

            }
        },
        j_online_countdown = {
            key = 'j_online_countdown',
            rarity = 4,
            order = 10,
            discovered = true,
            cost = 3,
            name = "Countdown!",
            pos = { x = 9, y = 9 },
            set = "Joker",
            config = {},
            perishable_compat = true,
            description = {
                main = "All opponent jokers became perishable (after 3 rounds)",
                sub = "Your opponent have to play a {X} to remove this effect at the end of the round"
            },
            can_activate_more_than_once_per_chain = false,
            on_debuff = function(card)
                G.ONLINE_CENTERS[card.config.center.key].cleanups['instant'](card.params.is_opponent)
            end,
            actions = {
                first_hand_drawn_joker = function(card)
                    if card.debuff then
                        return
                    end
                    play_sound('cardSlide1')
                    if not card.params.is_opponent then
                        if lume.all(G.opponent_jokers.cards, function(joker) return joker.debuff end) then
                            -- reset the countdown card
                            card.config.center.description.sub = get_center_for_online_card_by_name('j_online_countdown')
                                .description.sub
                            initialize_online_card(card)

                            card_eval_status_text(card, 'jokers', nil, nil, nil, {
                                message = localize('k_reset'),
                                colour = G.C.RED
                            })
                            play_sound('whoosh')
                        end

                        for _, joker in pairs(G.opponent_jokers.cards) do
                            joker:juice_up()
                            if not joker.ability.perishable then
                                joker:set_perishable(true)
                                joker.ability.perish_tally = 3
                            end
                        end
                    else
                        if lume.all(G.jokers.cards, function(joker) return joker.debuff end) then
                            -- reset the countdown card
                            card.config.center.description.sub = get_center_for_online_card_by_name('j_online_countdown')
                                .description.sub
                            initialize_online_card(card)

                            opponent_card_eval_status_text(card, 'jokers', nil, nil, nil, {
                                message = localize('k_reset'),
                                colour = G.C.RED
                            })
                            play_sound('whoosh')
                        end

                        for _, joker in pairs(G.jokers.cards) do
                            joker:juice_up()
                            if not joker.ability.perishable then
                                joker:set_perishable(true)
                                joker.ability.perish_tally = 3
                            end
                        end
                    end
                end,
                individual = function(card, _, game_context)
                    if not card.params.is_opponent then
                        if game_context.cardarea == G.opponent_play then
                            if game_context.other_card:get_id() == card.ability.countdown_id and game_context.other_card:is_suit(card.ability.countdown_suit) then
                                if not card.ability.countdown_should_remove then
                                    card.ability.countdown_should_remove = true
                                    card:set_debuff(true)
                                    card_eval_status_text(card, 'jokers', nil, nil, nil, {
                                        message = localize('k_debuffed'),
                                        colour = G.C.RED
                                    })
                                end

                                return true
                            end
                        end
                    else
                        if game_context.cardarea == G.play then
                            if game_context.other_card:get_id() == card.ability.countdown_id and game_context.other_card:is_suit(card.ability.countdown_suit) then
                                if not card.ability.countdown_should_remove then
                                    card.ability.countdown_should_remove = true
                                    card:set_debuff(true)
                                    opponent_card_eval_status_text(card, 'jokers', nil, nil, nil, {
                                        message = localize('k_debuffed'),
                                        colour = G.C.RED
                                    })
                                end

                                return true
                            end
                        end
                    end
                    return false
                end,
                end_of_round_after_jokers = function(card)
                    if card.ability.countdown_should_remove then
                        card:start_dissolve()
                    end
                end
            },
            cleanups = {
                instant = function(is_opponent)
                    if is_opponent then
                        for _, joker in pairs(G.jokers.cards) do
                            joker.ability.perishable = false
                        end
                    else
                        for _, joker in pairs(G.opponent_jokers.cards) do
                            joker.ability.perishable = false
                        end
                    end
                end
            }
        },
        j_online_discard_maestro = {
            key = 'j_online_discard_maestro',
            rarity = 4,
            order = 10,
            discovered = true,
            cost = 3,
            name = "Discard Maestro",
            pos = { x = 9, y = 9 },
            set = "Joker",
            config = {},
            perishable_compat = true,
            description = {
                main = "You can now discard more than 5 cards in your hand",
                sub = "Every X amount > 5 will randomly discard an opponent card"
            },
            can_activate_more_than_once_per_chain = false,
            on_debuff = function(card)
                G.ONLINE_CENTERS[card.config.center.key].cleanups['instant'](card.params.is_opponent)
            end,
            actions = {
                before_pre_discard = function(card, _, game_context)
                    if #game_context.full_hand <= 5 or card.debuff then
                        return
                    end

                    card:juice_up()

                    local to_discard = #game_context.full_hand - 5
                    local hand_to_discard = card.params.is_opponent and G.hand or G.opponent_hand
                    local discard_area = card.params.is_opponent and G.discard or G.opponent_discard
                    for i = 1, to_discard do
                        local random_card = custom_pseudorandom_element(hand_to_discard.cards,
                            opponent_pseudoseed('discard_maestro'))

                        if random_card then
                            draw_card(hand_to_discard, discard_area, i * 100 / to_discard, 'up', false, random_card)
                        end
                    end

                    -- We need to draw if the player hand if the opponent is discarding
                    -- The other way around is already handled by the update_draw_to_hand
                    if card.params.is_opponent then
                        G.E_MANAGER:add_event(Event({
                            trigger = 'after',
                            func = function()
                                G.FUNCS.draw_from_deck_to_hand(nil)

                                if G.buttons and G.buttons.REMOVED then
                                    on_resume_timer()
                                    G.E_MANAGER:add_event(Event({
                                        trigger = 'after',
                                        func = function()
                                            G.buttons = UIBox {
                                                definition = create_UIBox_buttons(),
                                                config = { align = "bm", offset = { x = 0, y = 0.3 }, major = G.hand, bond = 'Weak' }
                                            }
                                            return true
                                        end
                                    }))
                                end

                                return true
                            end
                        }))
                    end
                end
            },
            cleanups = {
                instant = function(is_opponent)
                    local hand_to_update_limit = is_opponent and G.opponent_hand or G.hand
                    local jokers_to_check = is_opponent and G.opponent_jokers.cards or G.jokers.cards

                    if not lume.any(jokers_to_check, function(c)
                            return c.config.center.key ==
                                'j_online_discard_maestro'
                        end) then
                        hand_to_update_limit.config.highlighted_limit = 5
                    end
                end
            }
        },
        j_online_planet_eater = {
            key = 'j_online_planet_eater',
            rarity = 4,
            order = 10,
            discovered = true,
            cost = 3,
            name = "Planet eater",
            pos = { x = 9, y = 9 },
            set = "Joker",
            config = {},
            perishable_compat = true,
            description = {
                main = "1 / 2 chance to change randomly a opponent planet card",
                sub = ""
            },
            can_activate_more_than_once_per_chain = true,
            actions = {
                before_using_consumeable = function(card, _, game_context)
                    if game_context.consumeable
                        and game_context.consumeable.config.center.set == 'Planet'
                        and game_context.consumeable.params.is_opponent ~= card.params.is_opponent
                        and pseudorandom('planet_eater') >= 0.5
                    then
                        card:juice_up()
                        G.E_MANAGER:add_event(Event({
                            trigger = 'after',
                            delay = 2.0,
                            func = function()
                                local all_planet_config = lume.filter(G.P_CENTERS, function(center)
                                    return center.set == 'Planet'
                                end)

                                delay(2.0)
                                local random_center = custom_pseudorandom_element(all_planet_config,
                                    opponent_pseudoseed('planet_eater'))
                                game_context.consumeable.config.center = random_center
                                game_context.consumeable.ability.consumeable = game_context.consumeable.config.center
                                    .config
                                game_context.consumeable:set_sprites(game_context.consumeable.config.center)
                                game_context.consumeable.ability.consumeable.hand_type = random_center.config.hand_type

                                game_context.consumeable:juice_up()
                                play_sound('multhit2')

                                delay(1.0)

                                return true
                            end
                        }))
                    end
                end
            },
            cleanups = {
            }
        }
    }
end

function register_online_interaction(owner)
    local center = get_online_center_config_by_name_or_key(owner.ability.name)
    if not center then
        return false
    end

    local interaction = Interaction:new(owner)
    for action_ctx, action in pairs(center.actions) do
        interaction:add_action(action_ctx, action)
    end

    BALATRO_VS_CTX.interaction_context:register_actions(interaction)

    return true
end

function get_online_center_config_by_name_or_key(name_or_key)
    for _, center in pairs(G.ONLINE_CENTERS) do
        if center.name == name_or_key or center.key == name_or_key then
            return lume.clone(center)
        end
    end
    return nil
end

function get_center_for_online_card_by_name(name) --Mostly the same as above but without the functions
    local center = get_online_center_config_by_name_or_key(name)
    if not center then
        print("Center not found for name: " .. name)
        return nil
    end
    return custom_copy_table(center)
end

function initialize_online_card(card)
    if card.config.center.key == "j_online_countdown" then
        local valid_cards = {}

        if card.params.is_opponent then
            for k, v in ipairs(G.opponent_playing_cards) do
                if v.ability.effect ~= 'Stone Card' then
                    valid_cards[#valid_cards + 1] = v
                end
            end
        else
            for k, v in ipairs(G.playing_cards) do
                if v.ability.effect ~= 'Stone Card' then
                    valid_cards[#valid_cards + 1] = v
                end
            end
        end

        table.sort(valid_cards, function(a, b) return a:get_nominal('suit') > b:get_nominal('suit') end)
        if valid_cards[1] then
            local random_card = custom_pseudorandom_element(valid_cards, opponent_pseudoseed('countdown'))
            card.ability.countdown_id = random_card:get_id()
            card.ability.countdown_suit = random_card.base.suit
            local text = ", " ..
                localize(random_card.base.value, 'ranks') .. " " ..
                localize(random_card.base.suit, 'suits_plural') .. " ,"
            card.config.center.description.sub = card.config.center.description.sub:gsub("{X}", text)
        else
            card.config.center.description.sub = "No valid card to play"
        end
    elseif card.config.center.key == "j_online_discard_maestro" then
        local hand_to_update_limit = card.params.is_opponent and G.opponent_hand or G.hand
        hand_to_update_limit.config.highlighted_limit = 100
    elseif card.config.center.key == "j_online_begone_joker" then
        card.ability.begone_cost = 15
    end
end

function get_online_center_by_key(key)
    return lume.clone(G.ONLINE_CENTERS[key])
end

function get_localized_description(key)
    local center = get_online_center_by_key(key)
    if not center then
        return ""
    end

    local description =
    {
        name_parsed = {
            {
                {
                    strings = {
                        center.name
                    },
                    control = {}
                }
            }
        },
    }

    return description
end

function is_online_card(key_or_name)
    if G.ONLINE_CENTERS[key_or_name] ~= nil then
        return true
    end

    for _, center in pairs(G.ONLINE_CENTERS) do
        if center.name == key_or_name then
            return true
        end
    end

    return false
end

function is_online_card_and_present(name)
    local is_online = is_online_card(name)

    if not is_online then
        return false
    end

    local is_present = false
    for _, card in pairs(G.jokers.cards) do
        if card.ability.name == name then
            is_present = true
        end
    end

    for _, card in pairs(G.opponent_jokers.cards) do
        if card.ability.name == name then
            is_present = true
        end
    end

    return is_present
end

local Interaction = {}
Interaction.__index = Interaction

function Interaction:new(owner)
    local self = {
        owner = owner,
        context_actions = {}
    }
    setmetatable(self, Interaction)

    return self
end

function Interaction:add_action(context, action)
    if not self.context_actions[context] then
        self.context_actions[context] = {}
    end

    table.insert(self.context_actions[context], action)
end

local InteractionManager = {}
InteractionManager.__index = InteractionManager

function InteractionManager:new()
    local self = {
        registered_actions = {},
        cleanups = {}
    }
    setmetatable(self, InteractionManager)

    return self
end

function InteractionManager:register_actions(action)
    table.insert(self.registered_actions, action)
end

function InteractionManager:cleanup_action(versus_center_id, name)
    local action = lume.filter(self.registered_actions,
        function(a) return a.owner.balatro_vs_center_id == versus_center_id and a.owner.ability.name == name end)[1]

    if action then
        lume.remove(self.registered_actions, action)
        local center = get_online_center_config_by_name_or_key(action.owner.ability.name)
        if center and center.cleanups then
            for ctx_key, cleanup in pairs(center.cleanups) do
                if ctx_key == "instant" then
                    if center.key == 'j_online_jester' or center.key == 'j_online_countdown' then
                        cleanup(action.owner.params.is_opponent)
                    else
                        cleanup()
                    end
                else
                    if not self.cleanups[ctx_key] then
                        self.cleanups[ctx_key] = {}
                    end

                    if center.key == 'c_online_negate_joker_round' and ctx_key == 'on_cashout' then
                        table.insert(self.cleanups[ctx_key],
                            function()
                                cleanup(action.owner.ability.targets)
                                return true
                            end)
                    else
                        table.insert(self.cleanups[ctx_key],
                            function()
                                cleanup()
                                return true
                            end)
                    end
                end
            end
        end

        return true
    end
    return false
end

function InteractionManager:handle_interaction(game_context, interaction_context)
    local handled_once_per_chain_interactions = {}
    for i, action in pairs(self.registered_actions) do
        if not handled_once_per_chain_interactions[action.owner.config.center.key] then
            if is_online_card(action.owner.config.center.key) and not G.ONLINE_CENTERS[action.owner.config.center.key].can_activate_more_than_once_per_chain and action.owner.config.center.set == 'Joker' then
                handled_once_per_chain_interactions[action.owner.config.center.key] = true
            end
            for ctx_key, ctx_val in pairs(game_context) do
                if ctx_val and action.context_actions[ctx_key] then
                    for ctx_name, act in pairs(action.context_actions[ctx_key]) do
                        --print("Executing action for owner: " .. action.owner.ability.name .. ", Context: " .. ctx_key)
                        act(action.owner, interaction_context, game_context)
                    end
                end
            end
        end
    end

    for ctx_key, ctx_val in pairs(game_context) do
        if ctx_val == true and self.cleanups[ctx_key] then
            for _, cleanup in pairs(self.cleanups[ctx_key]) do
                cleanup()
            end

            self.cleanups[ctx_key] = {}
        end
    end
end

return {
    InteractionContext = InteractionManager,
    Interaction = Interaction
}
